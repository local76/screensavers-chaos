# chaos

> The system logo, deconstructed. 7 explosion types, all driven by live system load.

A chaotic, particle-based screensaver centered on your live operating system logo. The big block logo in the middle of the screen (your actual OS name like "WIN11", "POPOS", "FEDORA", plus the kernel string underneath) is built from individual particles.

These particles normally sit in their correct "home" positions forming the logo. Periodically the logo becomes unstable and explodes in one of seven dramatic ways, with the particles flying around the screen before snapping back together.

## Animation phases

The screensaver cycles through four phases:

1. **Assembled**. The logo is stable. Particles are locked to their home positions. Background stars twinkle calmly.
2. **Exploding** (brief). Chooses the next explosion style and launches all particles outward / inward with initial velocity.
3. **Chaos**. Particles fly with physics specific to the current explosion type. They bounce off screen edges. Nearby stars get excited and display cinematic lens flares and starburst spikes.
4. **SnapBack**. Strong spring forces pull particles back to their logo positions. Once everything is home, it returns to Assembled.

## Explosion types (cycles automatically)

- **Supernova** — classic outward explosion with drag.
- **BlackHole** — particles are pulled inward (includes a secondary burst on longer stays).
- **Vortex** — swirling, spiraling motion.
- **GlitchWave** — digital chaos: strong horizontal waves + particles randomly glitch into binary, block characters, symbols, etc.
- **Shockwave** — powerful expanding pressure wave (strong radial push with wave modulation).
- **Entropy** — slow dissolution: particles get increasing random jitter and drift, with heavy character corruption (feels like digital decay).
- **Resonance** — vibrating / oscillating motion along radial lines from the center (particles hum back and forth).

## Dynamic / live behavior

- **Live logo**. The centered logo is fully dynamic. It pulls your real OS name (`logo_text`) and kernel version from the system at runtime via `get_system_info()`.
- **System load reactions**. Higher CPU/memory pressure makes explosions happen more frequently and intensely (shorter timers, stronger forces).
- **Per-machine personality**. A small `host_bias` derived from your hostname gives each computer subtle unique behavior.
- **Accent color**. Heavily tinted by your current Windows theme accent color (or the custom theme you set in the registry).
- **Background stars**. React to flying particles with lens flares and starbursts. Excitation level affects brightness and flare intensity.

## Configuration (registry)

Under `HKEY_CURRENT_USER\Software\local76\chaos`:

- `ParticleLimit` (0–2). Controls how many particles participate in the logo. 0 = some particles are randomly skipped (more unstable look).
- `ExplosionFreq` (0–2). Controls how often the logo explodes. Higher values = more frequent chaos.

Global settings (under `...\Settings`):

- `ColorTheme`: 0 = system accent, 1 = Cyan, 2 = Magenta, 3 = Green, 4 = Amber, 5 = Red.
- `GlobalScanlines`: 1 = enable scanlines effect on all scenes.

## Notes

- The logo text and kernel are pulled live every time the screensaver initializes or the screen size changes.
- This scene looks especially good on multi-monitor setups (it spans the full virtual screen).
- One of the first scenes to receive the full live system integration treatment.

## Technical

- Particles use velocity + damping + boundary bouncing.
- SnapBack uses spring physics with high damping to prevent oscillation.
- Lens flares and starbursts are drawn procedurally only on highly excited stars.

Part of the [screensavers](https://github.com/local76/screensavers) collection. See the root README for installation and the rest of the scenes.
