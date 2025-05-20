use crate::components::{Input_Teams,};
use dioxus::prelude::*;


enum PageState {
    Initial,
    Manual,
    CsvImport,
}

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn New_Draw() -> Element {

    let mut page_state = use_signal(|| PageState::Initial);

    rsx! {
        // I want to create a new draw landing page. Users can select between adding teams manually or loading a CSV file
        // The CSV file should have 2 cols: teamName, rank

        // I want a signal to remember what button was clicked so I can conditionally render the correct form

        div {
            id: "new-draw",



            {match *page_state.read() {
            PageState::Initial => rsx!(
                div {
                                // Content
                    h1 { "New Draw" }
                    p { "Please either enter teams and ranks manually, or select a CSV to import." }
                    
                    button {
                        onclick: move |_| page_state.set(PageState::Manual),
                        "Manually Enter Teams"
                        }
                    button {
                        onclick: move |_| page_state.set(PageState::CsvImport),
                        "Import CSV"
                        }
                    }
            ),
            PageState::Manual => rsx! {
                div {
                    h1 { "Input Teams " }
                }
                Input_Teams {}
            },
            PageState::CsvImport => rsx! {
                div {
                    h1 { "Upload Teams" }
                }
            }}}
        }
    }




}

