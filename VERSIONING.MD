# DeepX Kernel - Versioning Standard

This document defines the official versioning policy for the DeepX Kernel project. All releases and development builds must strictly adhere to these rules.

## 1. Version Format

The format follows a strict hierarchical structure:
`v<MAJOR>.<MINOR>.<PATCH>-<STAGE>.<BUILD>`

**Example:** `v0.1.0-alpha.1`

- **v**: Mandatory prefix.
- **Dots (.)**: Separators for version numbers and build increments.
- **Dash (-)**: Separator for the pre-release stage.
- **Lowercase**: All stage names must be lowercase.

## 2. Component Definitions

| Level     | Meaning                              | Impact                  |
| :-------- | :----------------------------------- | :---------------------- |
| **MAJOR** | Significant architectural changes    | Breaking / Incompatible |
| **MINOR** | New features, API additions          | Backward compatible     |
| **PATCH** | Bug fixes, optimization, refactoring | No behavioral change    |

## 3. Stage Evolution (Lifecycle)

A version must evolve through these stages in order:
`dev` → `alpha` → `beta` → `rc` → `Stable (no suffix)`

- **dev**: Internal builds, incomplete features, experimental code.
- **alpha**: Feature complete for the target version, but unstable. Public testing phase.
- **beta**: Feature freeze. Focus on stabilization and bug hunting.
- **rc (Release Candidate)**: Potential final build. If no critical bugs are found, it becomes Stable.
- **Stable**: Final, verified release.

## 4. Increment Rules

- **Build Number**: Increases with bug fixes, refactoring, or optimizations within the same stage (e.g., `alpha.1` → `alpha.2`).
- **Prohibited**: No hybrid stages (e.g., "DevAlpha"), no underscores, and no version bumping for aesthetic reasons.

## 5. Metadata (Optional)

Dates or commit hashes may be added as metadata using the `+` sign:
Example: `v0.1.0-alpha.1+2026.02`

## 6. When to Increment?

| Target    | Change Type                                                                                        | Action              |
| :-------- | :------------------------------------------------------------------------------------------------- | :------------------ |
| **PATCH** | Refactoring, bug fixes, or minor cleanups.                                                         | `v0.1.0` → `v0.1.1` |
| **MINOR** | New command added to Shell, new driver (e.g., RTC), or VFS extension.                              | `v0.1.x` → `v0.2.0` |
| **MAJOR** | Switching from BIOS to UEFI, changing the Task/Process structure, or replacing the Memory Manager. | `v0.x.x` → `v1.0.0` |

### Stage Transition Rules:

1. **To Alpha**: When all planned features for the version are implemented but untested.
2. **To Beta**: When no new features are added, and the system is stable enough for stress testing.
3. **To RC**: When all known bugs are fixed and the build is ready for "final check".
4. **To Stable**: When the RC has been tested without a single crash or regression.
