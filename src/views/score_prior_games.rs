use crate::components::{Score_Prior,};
use dioxus::prelude::*;

// static SONG: GlobalSignal<String> = Signal::global(|| "Drift Away".to_string());
// let mut team_s = use_signal(|| Vec::<TeamScore>::new());

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Score_Prior_Games() -> Element {

    
    rsx! {
        Score_Prior {}
        // Print_Results {}
        // Echo {}
    }
}
