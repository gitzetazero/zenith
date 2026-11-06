# Zenith Graphics: Video Settings

**Options > Video Settings...** opens the Zenith Graphics screen (the modern screen described in
[docs/graphics.md](../docs/graphics.md): search, pages, a cost and description for each setting,
presets and Undo). Vanilla's flat list is not shown by any menu; Done and Escape return to Options.
Upstream Horizon made this change itself (commit "Settings: each setting in one place, Video Settings
opens the graphics screen"); Zenith adds the checks and brand below on top of it.

![General page](docs-media/graphics-general.png)
![Quality page](docs-media/graphics-quality.png)

## Every vanilla Video Settings option

Keys are vanilla's `options.txt` keys, unchanged, so saves stay compatible and values clamp as in
vanilla's screen (the rows call the same option code).

| Vanilla option (`options.txt` key) | Where in Zenith Graphics | Status |
| --- | --- | --- |
| Preset: Fast, Fancy, Fabulous (`graphicsPreset`) | General > Preset, as **Vanilla Preset** (Zenith's own Preset is above it) | there; Zenith renames it so two rows don't both read "Preset" |
| Render Distance (`renderDistance`) | General > Distance | there |
| Simulation Distance (`simulationDistance`) | General > Distance | there |
| Brightness (`gamma`) | General > Distance | there |
| GUI Scale (`guiScale`) | General > Display | there; applies live |
| Fullscreen (`fullscreen`) | General > Display | there |
| Exclusive Fullscreen (`exclusiveFullscreen`) | General > Display | there |
| Exclusive Fullscreen Mode (`fullscreenResolution`) | General > Display | there |
| VSync (`enableVsync`) | General > Display | there |
| Max Framerate (`maxFps`) | General > Display | there |
| Reduce FPS when (`inactivityFpsLimit`) | General > Display | there |
| Attack Indicator (`attackIndicator`) | General > Interface | there |
| Autosave Indicator (`showAutosaveIndicator`) | General > Interface | there |
| Menu Background Blur (`menuBackgroundBlurriness`) | General > Interface | there |
| Graphics API (`preferredGraphicsBackend`) | Advanced > Graphics API | there (Zenith's Graphics API setting replaces vanilla's button) |
| Clouds (`renderClouds`), Cloud Range (`cloudRange`) | Quality > Sky and weather | there |
| Weather Radius (`weatherRadius`), Particles (`particles`) | Quality > Sky and weather | there |
| Smooth Lighting (`ao`), Biome Blend (`biomeBlendRadius`) | Quality > Lighting and entities | there |
| Entity Distance (`entityDistanceScaling`), Entity Shadows (`entityShadows`) | Quality > Lighting and entities | there |
| Vignette (`vignette`), Chunk Fade (`chunkSectionFadeInTime`) | Quality > Lighting and entities | there |
| Mipmap Levels (`mipmapLevels`) | Quality > Textures | there |
| Texture Filtering (`textureFiltering`), Max Anisotropy (`maxAnisotropyBit`) | Quality > Textures | there; Max Anisotropy greyed unless filtering is Anisotropic, as in vanilla |
| See-Through Leaves (`cutoutLeaves`), Improved Transparency (`improvedTransparency`) | Quality > Textures | there |
| Chunk Builder (`prioritizeChunkUpdates`) | Performance | there |
| FOV (`fov`), View Bobbing (`bobView`) | Options (FOV) and Accessibility (View Bobbing), where vanilla has them | not video settings in vanilla 26.3; not on this screen |

Nothing in vanilla's Video Settings is missing. Zenith's own pages (Performance, Shaders, Advanced,
HDR) add to it. Vanilla's own layout is still reachable for harnesses with
`zenith-ctl options video-vanilla`.

## What Zenith adds

- The mark over the search field reads Zenith (`zenith.settings.brand`, English and Spanish in
  `crates/zenith-client/assets/zenith/lang/`); upstream draws the literal "Horizon".
- Vanilla's preset row is named Vanilla Preset (`Vanilla Preset` in the same files).
- Tests in `crates/zenith-client-render/src/zenith_graphics_ui.rs`: the screen's texts never say
  Horizon, and the preset row's name.
