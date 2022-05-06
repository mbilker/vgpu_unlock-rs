# Rust-based vgpu\_unlock

Unlock vGPU functionality for consumer-grade NVIDIA GPUs.

**This tool is to be used with the kernel patches from the main
[`vgpu_unlock`](https://github.com/DualCoder/vgpu_unlock) repository!**

## Dependencies

* This tool requires Rust. You can install it via your package manager or via
  [](https://rustup.rs).
* Rust requires a linker to be installed to be able to create the shared
  library. Typically, this is installed with the C compiler through your
  distribution's package manager.
* The dependencies from the main `vgpu_unlock` project excluding Python and
  `frida`.

## Installation

In the following instructions `<path_to_vgpu_unlock_rs>` needs to be replaced
with the path to this repository on the target system.

Install the NVIDIA vGPU driver and kernel driver patches as detailed in the
main `vgpu_unlock` project README. Ignore the steps regarding editing the
systemd service unit files.

Run `cargo build --release` to compile the shared library.

Create the directories `/etc/systemd/system/nvidia-vgpud.service.d` and
`/etc/systemd/system/nvidia-vgpu-mgr.service.d`.

Create the files `/etc/systemd/system/nvidia-vgpud.service.d/vgpu_unlock.conf`
and `/etc/systemd/system/nvidia-vgpu-mgr.service.d/vgpu_unlock.conf`
with the following:
```
[Service]
Environment=LD_PRELOAD=<path_to_vgpu_unlock_rs>/target/release/libvgpu_unlock_rs.so
```

Create the directory `/etc/vgpu_unlock` which will house the vGPU profile
override configuration file.

Create the file `/etc/vgpu_unlock/profile_override.toml` with the profile
fields that are to be overridden. The following is an example for `nvidia-55`
(GRID P40-2A) that sets the number of heads to 1, sets the framebuffer to be
1920x1080 (1920 * 1080 = 2073600 pixels), enables CUDA, and disables the
frame-rate limiter.

```toml
[profile.nvidia-55]
num_displays = 1
display_width = 1920
display_height = 1080
max_pixels = 2073600
cuda_enabled = 1
frl_enabled = 0
```

If you want to enable VM migration or snapshotting, you must 
recompile the `nvidia-vgpu-vfio` kernel module with `NV_KVM_MIGRATION_UAPI` 
equal to 1. Then, create the file `/etc/vgpu_unlock/config.toml` and add the 
following:

```toml
unlock_migration = true
```

Happy hacking!
