# Linux Dynamic Wallpaper Platform Design

## Overview

This document defines the product blueprint for a new Linux desktop application that replaces the current `wayvid` direction. The new product is not a video-wallpaper utility with extra features. It is a Linux dynamic wallpaper platform designed primarily for Wallpaper Engine migration users.

The product goal is to provide the best Linux experience for discovering, acquiring, importing, understanding, and running Wallpaper Engine workshop wallpapers, with first-release emphasis on `video` and `scene` content types.

The working assumption is that the final product will use a new brand name rather than `wayvid`, with a descriptive subtitle that explains its purpose.

## Product Positioning

### What this product is

- A Linux dynamic wallpaper platform.
- A desktop application first, not a daemon-first system.
- A migration path for Wallpaper Engine users on Linux.
- A compatibility-focused product built around Steam Workshop content.

### What this product is not

- Not primarily a generic local wallpaper manager.
- Not primarily a video wallpaper player.
- Not a content editor or wallpaper creation suite.
- Not a replacement for Steam's authorization or distribution systems.

## Primary Users

### Primary audience

Wallpaper Engine migration users on Linux.

These users already understand Workshop content and care most about whether their preferred wallpapers can be used on Linux with minimal friction.

### Secondary audiences

- General desktop customization users who want a polished, easy wallpaper experience.
- Advanced Linux users who care about multi-monitor behavior, performance, runtime control, and integration quality.

## Core Product Promise

The product should make the following promise credible:

> On Linux, I can browse Wallpaper Engine workshop content, acquire it through official Steam mechanisms, understand what is compatible, and use it in a polished daily desktop experience.

This is stronger than "can play video wallpapers" and narrower than "rebuild all of Wallpaper Engine immediately." It gives the product a clear wedge while keeping the first release focused.

## First-Release Strategy

### Product path

The recommended first-release strategy is a dual-core path:

1. Deliver a real Workshop compatibility loop.
2. Deliver a mature daily-use desktop application.

This avoids two failure modes:

- a strong technical prototype that is not pleasant to use daily
- a polished wallpaper app that lacks a strong differentiator

### Content-type strategy

The platform model should recognize three first-class wallpaper content types:

- `video`
- `scene`
- `web`

For the first release:

- `video` is a primary supported type
- `scene` is a primary supported type
- `web` is recognized in the product model and UI, but not a first-release focus for strong runtime support

This keeps the long-term product definition correct without forcing the first release to solve the hardest runtime and security problems at once.

## Information Architecture

### Top-level navigation

Recommended top-level structure:

- `Library`
- `Workshop`
- `Desktop`
- `Settings`

### Library

`Library` is the default landing page.

It should center the user's owned or synchronized wallpaper collection and support:

- browsing acquired content
- search, filter, and sort
- favorites and recents
- compact compatibility status visibility
- lightweight current desktop status visibility

The page should feel like a personal wallpaper control center, not a storefront and not a diagnostics dashboard.

### Workshop

`Workshop` is a first-class destination but not the home page.

It should support:

- browsing and search
- category and tag navigation
- item details
- acquire/subscribe flow initiation
- sync progress visibility
- compatibility visibility before and after acquisition

### Desktop

`Desktop` should support active-use operations:

- current display/output overview
- currently applied wallpapers
- multi-monitor assignment
- runtime status
- quick playback-related controls when appropriate

### Settings

`Settings` should cover:

- Steam integration
- language selection
- compatibility and runtime options
- performance options
- startup/background behavior

## Home Page Behavior

The home page should be `Library`, not `Workshop` and not a split dashboard.

Rationale:

- daily use centers on the user's collection, not content discovery every time
- a split home page creates visual crowding and weakens both sides
- a desktop app should feel grounded in ownership and control, not in feed-like discovery

The `Library` page may include a compact current-desktop status module, but it must not dominate layout or compete with the wallpaper grid.

## Workshop Integration Model

### Core principle

The first release should strongly depend on the local Steam client being installed and logged in.

This is acceptable because Workshop compatibility is the core competitive value of the product, not an optional add-on.

### Experience goal

The experience goal is in-app acquisition with minimal friction:

1. Browse workshop content in the app.
2. Initiate acquisition/subscription from the app.
3. Use official Steam mechanisms for authorization and distribution.
4. Detect synchronization/download completion.
5. Import into the local library.
6. Surface compatibility status.
7. Preview and apply.

### Platform boundary

The product should not define itself as an alternative downloader or distributor for Workshop content.

The intended boundary is:

- Steam handles authorization and subscription/distribution.
- The application handles discovery, orchestration, synchronization awareness, import, compatibility analysis, and runtime behavior.

This boundary keeps the product closer to a compatibility layer and desktop application than to a replacement distribution channel.

## Compatibility Strategy

Compatibility is a user-facing product capability, not just an internal technical detail.

Every relevant wallpaper item should expose a visible compatibility level:

- `Fully Supported`
- `Partially Supported`
- `Experimental`
- `Unsupported`

The product should also explain, where possible:

- why a wallpaper has that status
- what features are missing
- whether there is a usable fallback path
- what the user can try next

This is especially important for migration users, because trust depends not only on running content but also on clearly communicating limitations.

## Required First-Release Capabilities

The first release must include the following capability groups.

### 1. In-app Workshop loop

- browse Workshop content in-app
- inspect item details in-app
- trigger official acquisition/subscription flow
- detect synchronization completion
- import synchronized content into the library

### 2. Content-type recognition

- identify `video`, `scene`, and `web`
- present content type clearly in UI
- attach compatibility status to each relevant item

### 3. Runtime support with first-release focus

- usable preview/apply flow for `video`
- usable preview/apply flow for `scene`
- `web` recognized in the system even when strong runtime support is deferred

### 4. Library-centered daily use

- default `Library` home page
- search, filter, sort
- favorites and recents
- compact status indicators for compatibility and updates

### 5. Desktop integration

- multi-monitor assignment
- background operation
- restore previous session/state
- basic runtime/performance controls

### 6. Internationalization

- first release must support both Chinese and English
- language choice must be explicit and user-controlled
- compatibility explanations, runtime errors, and settings text must all be localizable
- the i18n system must be designed so future languages can be added without UI redesign or text-system rewrites

## Explicitly Deferred First-Release Capabilities

The first release should explicitly defer the following.

### Deferred runtime depth

- strong `web` wallpaper support
- advanced support for edge-case or highly custom content types beyond the main roadmap

### Deferred product surfaces

- wallpaper editing or authoring tools
- creator workflows
- account systems
- cloud sync
- community/social features such as comments or ratings

### Deferred power-user systems

- advanced rule automation based on time, workspace, battery, context, or scenes
- plugin systems
- generalized extensibility systems introduced before core models stabilize

### Deferred scope expansion

- non-Linux platform support
- turning the app into a generic media asset manager unrelated to the Workshop-centered story

## Product Style Direction

The visual and interaction direction should be modern and restrained.

It should borrow the clarity and information structure users expect from Wallpaper Engine-like tools, but avoid becoming visually crowded or technically intimidating.

That means:

- clear navigation
- dense enough information for confidence
- not overusing dashboard or status-panel layouts
- strong focus on content browsing and control
- a desktop feel rather than a web portal feel

## Naming Direction

The product should use a new abstract brand name rather than continuing with `wayvid`.

Reasons:

- `wayvid` is too strongly tied to video
- the new product is broader than video wallpapers
- an abstract name gives better long-term room for `video`, `scene`, and `web` support

The recommended external naming structure is:

- abstract product name
- descriptive subtitle/tagline that makes the category obvious

## Repository Reset Strategy

The new product should not be treated as a light refactor of the current repository.

It should be treated as a repository reset built around a new product definition.

### Principles

- keep only high-value technical assets
- remove legacy product framing aggressively
- rebuild project metadata and documentation around the new product story
- avoid carrying forward old naming, old architecture assumptions, and old docs by inertia

### OpenSpec reset

`openspec/config.yaml` should be rewritten to describe the new product context rather than the current `wayvid` frame.

That rewrite should include:

- the new product purpose
- Linux desktop application focus
- Steam/Workshop-centered positioning
- first-release focus on `video + scene`
- bilingual-first product expectations
- any rules for proposal/design/task artifacts that reflect the new repo direction

### Documentation reset

Most existing documentation in `docs/` should be assumed stale unless explicitly retained.

Recommended approach:

- archive or remove docs tied to the old product story
- rewrite top-level documentation from the new product promise outward
- only migrate factual material that still applies to the new product

### Code reset

Most existing code should be evaluated as migration candidates, not assumed foundations.

Likely migration candidates:

- core playback capabilities with proven value
- parts of Workshop parsing/import knowledge
- selected low-level Linux/Wayland runtime knowledge

Likely discard candidates:

- old product framing
- legacy UI structure
- duplicated config models
- transitional daemon-vs-GUI abstractions that no longer serve the new product

## First-Release Success Criteria

The first release succeeds if all three of these are true.

### 1. Migration credibility

Wallpaper Engine users on Linux feel this is a real candidate to become their primary daily solution.

### 2. Compatibility trust

Substantial `video + scene` content can be discovered, acquired, synchronized, imported, and run, with compatibility levels communicated clearly enough to build user trust.

### 3. Daily-use maturity

The application feels like a reliable desktop app that users can keep installed, keep running, and return to regularly, rather than a one-off technical demonstration.

## Recommended Next Step

The next step after approving this design is not implementation directly. It is to turn this product blueprint into an implementation plan that decomposes work into phases such as repository reset, naming/branding placeholder updates, Workshop loop design, compatibility model design, runtime architecture, and first-release UI surface planning.
