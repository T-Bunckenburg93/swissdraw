use crate::components::{Echo, Hero,Load_Results};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Calculate_Scores() -> Element {
    rsx! {
        Load_Results {}
        // Echo {}
    }
}
