# Notes

Ideas and measurements that are not on the phase plan yet. See [PLAN.md](PLAN.md) for the plan.

## Binary size: 20 MB release exe (2026-11-25)

v0.1.0 Windows release, stripped, is 20.1 MB. Unstripped it was 28 MB. The code
is tiny. The weight is Slint's runtime plus the `windows` crates pulled in by
`tray-icon` and `auto-launch`.

Untried, expected to roughly halve the size:

```toml
[profile.release]
strip = true          # already set
lto = true            # whole-program link-time optimization, drops unused code across crates
codegen-units = 1     # better optimization, slower compile
opt-level = "z"       # optimize for size instead of speed
panic = "abort"       # no unwinding tables
```

Costs:

- Release build gets slower. The 11 min cold build may become 15 to 20 min.
- `opt-level = "z"` can make the software renderer slower. The pill repaints once a
  second, so this should not matter. Measure CPU after.
- `panic = "abort"` kills the process on any panic instead of unwinding. Acceptable
  here, there is nothing to clean up.

Beyond the profile:

- Slint features: check which of `std`, `compat-1-2`, `backend-winit`,
  `renderer-software` can go. `backend-winit` could narrow to a per-OS backend.
- `sysinfo` has feature flags for components, disks, networks and processes. Only
  memory is needed. Disabling the rest trims code.
- Disk size is not RAM. Idle RSS is the number that matters, and it is still
  unmeasured until the exe runs on a Windows machine.

Try the profile change first, rebuild, compare the exe size and idle RSS, then
decide.
