//! Zenith: brand checks for the graphics screen (Video Settings), which upstream's own tests
//! don't cover: nothing it shows may say Horizon.
#[cfg(test)]
mod tests {
    use crate::{
        graphics_ui::{self, Item},
        lang,
        settings::{Opt, Settings},
    };

    #[test]
    fn vanillas_preset_row_is_named_apart_from_zeniths() {
        let label = Item::Vanilla(Opt::Preset)
            .label(&Settings::default())
            .plain();
        assert_eq!(label, "Vanilla Preset: options.graphics.fancy");
    }

    #[test]
    fn the_graphics_screen_never_says_horizon_in_english_or_spanish() {
        for code in ["en_us", "es_es", "de_de"] {
            let (shown, _) = lang::scoped(lang::Language::load(&[], code), || {
                let mut shown = vec![lang::get_or("zenith.settings.brand", "Zenith")];
                for (page, groups) in graphics_ui::rows() {
                    shown.push(lang::get(page.name()));
                    shown.push(lang::get(page.about()));
                    for (group, rows) in groups {
                        shown.push(lang::get(group));
                        for row in rows {
                            shown.push(lang::get(row.text));
                        }
                    }
                }
                shown
            });
            assert_eq!(shown[0], "Zenith", "{code}");
            let bad: Vec<_> = shown.iter().filter(|t| t.contains("Horizon")).collect();
            assert!(bad.is_empty(), "{code}: {bad:?}");
        }
    }
}

// Every vanilla Video Settings option is on the graphics screen: upstream's own test
// (`vanilla_video_rows_work_on_the_graphics_screen`) covers how they behave; this one covers
// that none is missing.
#[cfg(test)]
mod coverage {
    use crate::{
        graphics_ui::{self, Item},
        settings::Opt::{self, *},
    };

    const VANILLA_VIDEO: [Opt; 29] = [
        ExclusiveMode,
        MaxFps,
        Vsync,
        InactivityFps,
        GuiScale,
        Fullscreen,
        ExclusiveFullscreen,
        Gamma,
        Preset,
        BiomeBlend,
        RenderDistance,
        ChunkBuilder,
        SimulationDistance,
        SmoothLighting,
        Clouds,
        Particles,
        Mipmaps,
        EntityShadows,
        EntityDistance,
        MenuBlur,
        CloudRange,
        CutoutLeaves,
        ImprovedTransparency,
        TextureFiltering,
        MaxAnisotropy,
        WeatherRadius,
        Autosave,
        Vignette,
        AttackIndicator,
    ];

    #[test]
    fn no_vanilla_video_option_is_missing_from_the_graphics_screen() {
        let items: Vec<Item> = graphics_ui::rows()
            .into_iter()
            .flat_map(|(_, groups)| groups)
            .flat_map(|(_, rows)| rows)
            .map(|r| r.item)
            .collect();
        for opt in VANILLA_VIDEO.into_iter().chain([Vignette, ChunkFade]) {
            assert!(items.contains(&Item::Vanilla(opt)), "{opt:?}");
        }
    }
}
