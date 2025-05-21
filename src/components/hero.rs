use dioxus::prelude::*;
use crate::Route;

const HEADER_SVG: Asset = asset!("/assets/icon_transparent.png", ImageAssetOptions::new().with_avif());

#[component]
pub fn Hero() -> Element {
    rsx! {
        // We can create elements inside the rsx macro with the element name followed by a block of attributes and children.
        div {
            // Attributes should be defined in the element before any children
            id: "hero",
            // After all attributes are defined, we can define child elements and components
            img { src: HEADER_SVG, id: "header", style: "width: 40%; height: auto;" }
            div { id: "links",
                // The RSX macro also supports text nodes surrounded by quotes
                a { href: "https://dioxuslabs.com/learn/0.6/", "📚 Learn Dioxus" }
                Link { to: Route::New_Draw {}, "New Draw" }
                Link { to: Route::Load_Draw {}, "Load Draw" }
                Link { to: Route::Score_Prior_Games {}, "Score Results" }
            }
        }
    }
}
