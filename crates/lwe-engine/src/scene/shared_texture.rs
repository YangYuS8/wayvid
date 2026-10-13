//! One engine/context owns readonly uploads; renderers own leases, never cache pins.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use anyhow::{Context, Result, ensure};
use khronos_egl as egl;

use super::renderer::{GpuTexture, check_gl};

pub(crate) const PAYLOAD_LIMIT: u64 = 512 * 1024 * 1024;
const MAX_CACHE_ENTRIES: usize = 8192;
static NEXT_DOMAIN: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TextureKey {
    pub fingerprint: [u8; 32],
    pub repeat: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ResourcePlan {
    pub textures: BTreeMap<TextureKey, u64>,
    pub private_bytes: u64,
}

impl ResourcePlan {
    pub fn merge(&mut self, other: &Self) -> Result<()> {
        let private_bytes = self
            .private_bytes
            .checked_add(other.private_bytes)
            .context("Scene private allocation total overflow")?;
        for (key, bytes) in &other.textures {
            if let Some(previous) = self.textures.get(key) {
                ensure!(
                    *previous == *bytes,
                    "Shared Scene texture allocation sizes disagree"
                );
            }
        }
        self.private_bytes = private_bytes;
        self.textures
            .extend(other.textures.iter().map(|(key, bytes)| (*key, *bytes)));
        Ok(())
    }

    pub fn immutable_bytes(&self) -> Result<u64> {
        self.textures.values().try_fold(0_u64, |total, bytes| {
            total
                .checked_add(*bytes)
                .context("Scene immutable allocation total overflow")
        })
    }

    pub fn total(&self, retired: u64) -> Result<u64> {
        self.immutable_bytes()?
            .checked_add(self.private_bytes)
            .and_then(|total| total.checked_add(retired))
            .context("Scene logical allocation total overflow")
    }

    pub fn preflight(&self, retired: u64, additional_private: u64) -> Result<u64> {
        ensure!(
            self.textures.len() <= MAX_CACHE_ENTRIES,
            "Scene readonly texture entry budget exhausted"
        );
        let total = self
            .total(retired)?
            .checked_add(additional_private)
            .context("Scene replacement allocation total overflow")?;
        ensure!(
            total <= PAYLOAD_LIMIT,
            "Scene candidate and retained output resources exceed the engine's 512 MiB logical texture/framebuffer budget"
        );
        Ok(total)
    }
}

struct Allocation {
    bytes: u64,
    retired: Option<u32>,
}

#[derive(Default)]
struct Ledger {
    next: u64,
    allocations: BTreeMap<u64, Allocation>,
}

pub(super) struct SharedTexture {
    pub texture: GpuTexture,
    serial: u64,
    ledger: Rc<RefCell<Ledger>>,
}

impl Drop for SharedTexture {
    fn drop(&mut self) {
        // GL is deliberately absent from last-lease drop. The owning context
        // drains this serial before destroying its surface/context.
        let id = std::mem::replace(&mut self.texture.id, 0);
        if let Some(allocation) = self.ledger.borrow_mut().allocations.get_mut(&self.serial) {
            allocation.retired = Some(id);
        }
    }
}

pub(crate) struct TexturePool {
    generation: u64,
    egl: egl::DynamicInstance<egl::EGL1_4>,
    native_context: Cell<Option<egl::Context>>,
    owner: RefCell<Option<std::sync::Weak<AtomicBool>>>,
    closed: Cell<bool>,
    faulted: Cell<bool>,
    entries: RefCell<BTreeMap<TextureKey, Weak<SharedTexture>>>,
    ledger: Rc<RefCell<Ledger>>,
}

impl TexturePool {
    pub fn new() -> Result<Rc<Self>> {
        let mut generation = NEXT_DOMAIN.load(Ordering::Relaxed);
        loop {
            let next = generation
                .checked_add(1)
                .context("Scene context generation exhausted")?;
            match NEXT_DOMAIN.compare_exchange_weak(
                generation,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => generation = current,
            }
        }
        Ok(Rc::new(Self {
            generation,
            // Loading entry points does not bind a context or create a backend.
            egl: unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required() }
                .context("Load Scene texture context identity")?,
            native_context: Cell::new(None),
            owner: RefCell::new(None),
            closed: Cell::new(false),
            faulted: Cell::new(false),
            entries: RefCell::new(BTreeMap::new()),
            ledger: Rc::new(RefCell::new(Ledger::default())),
        }))
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn bind_owner(&self, context: &crate::egl::EglContext) -> Result<()> {
        let (native, lifetime) = context.scene_context_identity();
        let owner = self.owner.borrow().clone();
        if let Some(previous) = owner {
            ensure!(
                previous.ptr_eq(&lifetime),
                "Cannot replace a Scene texture context owner"
            );
        } else {
            ensure!(
                self.native_context
                    .get()
                    .is_none_or(|expected| expected == native),
                "Cannot replace a Scene texture context generation"
            );
            self.native_context.set(Some(native));
            *self.owner.borrow_mut() = Some(lifetime);
        }
        Ok(())
    }

    pub fn ensure_current(&self) -> Result<()> {
        ensure!(
            !self.closed.get() && !self.faulted.get(),
            "Scene texture domain is unavailable"
        );
        if let Some(owner) = self.owner.borrow().as_ref() {
            ensure!(
                owner
                    .upgrade()
                    .is_some_and(|alive| alive.load(Ordering::Acquire)),
                "Scene texture context owner has been destroyed"
            );
        }
        let current = self
            .egl
            .get_current_context()
            .context("Scene texture operation requires a current EGL context")?;
        if let Some(expected) = self.native_context.get() {
            ensure!(
                current == expected,
                "Scene texture belongs to a different EGL context generation"
            );
        } else {
            self.native_context.set(Some(current));
        }
        Ok(())
    }

    pub(super) fn acquire(
        &self,
        key: TextureKey,
        bytes: u64,
        upload: impl FnOnce() -> Result<GpuTexture>,
    ) -> Result<Rc<SharedTexture>> {
        self.ensure_current()?;
        self.drain_retired()?;
        self.entries
            .borrow_mut()
            .retain(|_, entry| entry.strong_count() != 0);
        if let Some(texture) = self.entries.borrow().get(&key).and_then(Weak::upgrade) {
            ensure!(
                self.ledger
                    .borrow()
                    .allocations
                    .get(&texture.serial)
                    .is_some_and(|allocation| allocation.bytes == bytes),
                "Shared Scene texture allocation sizes disagree"
            );
            return Ok(texture);
        }
        ensure!(
            self.entries.borrow().len() < MAX_CACHE_ENTRIES,
            "Scene texture cache entry budget exhausted"
        );
        let resident = self.resident_bytes()?;
        ensure!(
            resident
                .checked_add(bytes)
                .is_some_and(|total| total <= PAYLOAD_LIMIT),
            "Scene readonly texture allocation exceeds the payload budget"
        );
        let serial = self
            .ledger
            .borrow()
            .next
            .checked_add(1)
            .context("Scene texture allocation identity exhausted")?;
        let texture = upload()?;
        let result = Rc::new(SharedTexture {
            texture,
            serial,
            ledger: Rc::clone(&self.ledger),
        });
        let mut ledger = self.ledger.borrow_mut();
        ledger.next = serial;
        ledger.allocations.insert(
            serial,
            Allocation {
                bytes,
                retired: None,
            },
        );
        drop(ledger);
        self.entries
            .borrow_mut()
            .insert(key, Rc::downgrade(&result));
        Ok(result)
    }

    pub fn resident_bytes(&self) -> Result<u64> {
        self.ledger
            .borrow()
            .allocations
            .values()
            .try_fold(0_u64, |total, item| {
                total
                    .checked_add(item.bytes)
                    .context("Scene resident texture total overflow")
            })
    }

    pub fn retired_bytes(&self) -> Result<u64> {
        self.ledger
            .borrow()
            .allocations
            .values()
            .filter(|item| item.retired.is_some())
            .try_fold(0_u64, |total, item| {
                total
                    .checked_add(item.bytes)
                    .context("Scene retired texture total overflow")
            })
    }

    pub fn allocation_count(&self) -> u64 {
        self.ledger.borrow().allocations.len() as u64
    }
    pub fn lease_count(&self) -> u64 {
        self.entries
            .borrow()
            .values()
            .map(|entry| entry.strong_count() as u64)
            .sum()
    }

    pub fn drain_retired(&self) -> Result<()> {
        let retired: Vec<_> = self
            .ledger
            .borrow()
            .allocations
            .iter()
            .filter_map(|(serial, item)| item.retired.map(|id| (*serial, id)))
            .collect();
        if retired.is_empty() {
            return Ok(());
        }
        self.ensure_current()?;
        let ids: Vec<_> = retired.iter().map(|(_, id)| *id).collect();
        unsafe {
            gl::DeleteTextures(ids.len() as i32, ids.as_ptr());
        }
        if let Err(error) = check_gl("deleting retired shared Scene textures") {
            // Do not allocate/retry deletion after an ambiguous GL failure: an
            // old numeric ID might otherwise refer to a subsequent allocation.
            self.faulted.set(true);
            return Err(error);
        }
        let mut ledger = self.ledger.borrow_mut();
        for (serial, _) in retired {
            ledger.allocations.remove(&serial);
        }
        drop(ledger);
        self.entries
            .borrow_mut()
            .retain(|_, entry| entry.strong_count() != 0);
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        let result = if self.lease_count() != 0 {
            Err(anyhow::anyhow!(
                "Scene texture context closed with live leases; allocations retained until context teardown"
            ))
        } else {
            self.drain_retired()
        };
        self.closed.set(true);
        if result.is_ok() {
            self.entries.borrow_mut().clear();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destroyed_context_lifetime_rejects_even_a_recycled_native_handle() {
        let pool = TexturePool::new().unwrap();
        let lifetime = std::sync::Arc::new(AtomicBool::new(false));
        *pool.owner.borrow_mut() = Some(std::sync::Arc::downgrade(&lifetime));
        assert!(
            pool.ensure_current()
                .unwrap_err()
                .to_string()
                .contains("owner has been destroyed")
        );
        drop(lifetime);
        assert!(pool.ensure_current().is_err());
    }

    fn plan(identity: u8, repeat: bool, bytes: u64, private: u64) -> ResourcePlan {
        ResourcePlan {
            textures: BTreeMap::from([(
                TextureKey {
                    fingerprint: [identity; 32],
                    repeat,
                },
                bytes,
            )]),
            private_bytes: private,
        }
    }

    #[test]
    fn union_charges_shared_once_and_private_generations_separately() {
        let mut combined = plan(1, false, 400, 30);
        combined.merge(&plan(1, false, 400, 40)).unwrap();
        combined.merge(&plan(1, false, 400, 50)).unwrap();
        assert_eq!(combined.total(0).unwrap(), 520);
        combined.merge(&plan(1, true, 400, 0)).unwrap();
        assert_eq!(combined.total(7).unwrap(), 927);
        let before = combined.total(7).unwrap();
        assert!(combined.merge(&plan(1, false, 401, 0)).is_err());
        assert_eq!(combined.total(7).unwrap(), before);
    }

    #[test]
    fn preflight_accounts_retirement_and_full_resize_peak_before_allocation() {
        let combined = plan(1, false, PAYLOAD_LIMIT - 20, 10);
        assert_eq!(combined.preflight(5, 5).unwrap(), PAYLOAD_LIMIT);
        assert!(combined.preflight(5, 6).is_err());
        assert!(plan(1, false, u64::MAX, 1).preflight(0, 0).is_err());
        let mut overflow = ResourcePlan {
            private_bytes: u64::MAX,
            ..ResourcePlan::default()
        };
        assert!(
            overflow
                .merge(&ResourcePlan {
                    private_bytes: 1,
                    ..ResourcePlan::default()
                })
                .is_err()
        );
    }
}
