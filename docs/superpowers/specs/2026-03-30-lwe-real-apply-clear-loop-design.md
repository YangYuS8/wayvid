# LWE Real Apply/Clear Loop Design

## Goal

Prove that LWE is not just a structured shell by implementing and validating one real end-to-end desktop action loop on the current local machine:

`Library -> Apply to a real monitor -> Desktop reflects the result -> Clear`

## Core Principle

This work is only successful if it is **validated on the current local environment**.

That means the goal is not:

- “the commands exist”
- “the UI is wired up”
- “the architecture looks right”

The goal is:

- a real apply operation happens on the user’s machine
- Desktop state reflects it
- a real clear operation happens afterward

## Scope

### In scope

- pick one real apply path that can work on the current machine
- select a real monitor target
- perform a real apply action from Library
- refresh Desktop so it reflects the result
- perform a real clear action from Desktop
- manually verify the full loop on the local environment

### Out of scope

- all-monitor apply
- preview mode
- advanced multi-monitor rules
- broad runtime redesign
- claiming support for every content type
- broad restore polish

## Product Framing

The purpose of this work is to move LWE from “architecturally promising” to “demonstrably usable.”

It should establish one reliable, real interaction path before broader capability expansion.

## Page Responsibilities

### Library

Library remains the place where the user chooses content and initiates application:

- choose one item
- choose one real monitor
- apply it
- see assignment feedback for that item

### Desktop

Desktop remains the place where the user observes current output state and clears it:

- see the applied state reflected
- clear the current assignment

The Desktop page is not expanded into a broader control console in this phase.

## Real-Environment Constraint

This design explicitly depends on the current local environment.

The implementation should prefer:

- the simplest real monitor path available on the current machine
- the most reliable currently supported content path for validation
- local manual verification as part of the success criteria

This is not the phase for maximizing generality. It is the phase for proving reality.

## Success Criteria

This work is successful only if all of the following are true:

1. A Library item can be selected and targeted to a real monitor.
2. The apply operation causes a real visible/system-level change on the current machine.
3. The Desktop page reflects the current assigned state after apply.
4. A clear action removes that state again.
5. The full loop is manually verified on the local environment.

## Implementation Strategy

The implementation should be intentionally narrow.

Recommended approach:

1. Identify the most likely-to-succeed real content/apply path in the current environment.
2. Make apply/clear truthful and backend-connected for that path.
3. Keep page-state synchronization tight between Library and Desktop.
4. Verify the loop manually on the local machine.

## Non-Goal Reminder

This is not the phase for proving that LWE already supports all wallpaper/runtime scenarios.

It is the phase for proving that at least one real Desktop workflow is actually operational.
