use dioxus::prelude::*;
use swissdraw::{SwissDraw,Team,Game};
use crate::DB;
use rusqlite::{Connection, Result};

const HEADER_SVG: Asset = asset!("/assets/icon_transparent.png", ImageAssetOptions::new().with_avif());


// I want a component where I enter the teams and ranks manually, and they are saved into a Vec of teams. 
// This vec is displayed, and the user can them submit them into a swiss draw
#[component]
pub fn Input_Teams() -> Element {

    let mut teams = use_signal(|| Vec::<Team>::new());
    // Team::new(rand::random(), name, rank);

    let mut team_name = use_signal(|| "".to_string());
    let mut team_rank = use_signal(|| "0".to_string());
    let mut sd_name = use_signal(|| "".to_string());

    rsx! {
        div {
            id: "input-teams",
            p { "Please enter the teams and their ranks below." }
            // I want to create a form where the user can enter the team name and rank
            form {
                onsubmit: move |evt| {
                        // I want to add the team to the teams vec
                        evt.prevent_default();
                    if team_name().is_empty() {
                            // Don't proceed if the input is empty
                            // log::warn!("Form is empty!");
                            return;
                        } else {
                            teams
                            .write()
                            .push(Team::new(rand::random(),team_name.to_string(),team_rank().parse().unwrap()));
                            // I want to display the teams in a table
                            println!("Teams: {:?}", teams.read());
                            // I want to clear the input fields
                            team_name.set("".to_string());
                            team_rank.set("0".to_string());
                        }
                    },
                input { 
                    type: "text", 
                    placeholder: "Team Name", 
                    value: "{team_name}", 
                    oninput: move |event| {team_name.set(event.value());}
                }
                input { 
                    type: "number", 
                    placeholder: "Rank", 
                    value: "{team_rank}", 
                    oninput: move |event| {team_rank.set(event.value());}
                },
                input { r#type: "submit" }
            }
            // I want a bit of a space between the form and the table
            br {}
            
            // I want to display the teams in a table

            table {
                thead {
                    tr {
                        th { "Team Name" }
                        th { "Inital Rank" }
                    }
                }
                tbody {
                    {teams.iter().map(|team| {

                            let formatted_score = format!("{:.1}", team.rank);
                            rsx! {
                                tr {
                                    td { class: "data-cell", "{team.name}" }
                                    td { class: "data-cell", "{formatted_score}" }
                                }
                            }
                        })
                    }
                }
            }

            br {}

    // I want to create a button that will submit the teams into a swiss draw
// The logic will come in a bit
// The button will save the teams into a swiss draw. write them into the db, and then redirect to the draw page with the id of the draw
// The button will only appears if there are at least 2 teams in the vec
            if teams.read().len() > 1 {

                p { "Please enter a name for the swiss draw." }
                input { 
                    type: "text", 
                    placeholder: "Swiss Draw Name", 
                    value: "{sd_name}", 
                    oninput: move |event| {sd_name.set(event.value());}
                }
                button {
                    onclick: move |evt| {
                        evt.prevent_default();

                        if sd_name().is_empty() {
                            // Don't proceed if the input is empty
                            // log::warn!("Form is empty!");
                            return;
                        } else {

                        // I want to create a new swiss draw
                        let mut sd = SwissDraw::new();
                        sd.round = 1;
                        sd.name = sd_name.to_string();
                        sd.team_list = teams.read().clone();
                        // I want to save the swiss draw into the db
                        // sd.save_to_db();
                        // I want to redirect to the draw page with the id of the draw
                        // Redirect to the draw page with the id of the draw

                        let sd_len = sd.team_list.len();
                        println!("Swiss Draw: {:?}", sd.team_list);
                        println!("Swiss Draw name: {:?}", sd.name);

                        let conn = DB.lock().unwrap();
                        sd.sync_draw(&conn).expect("Failed to sync draw");

                        }
                    },
                    "Submit Teams"
                }
            } else {
                 p { "Please enter at least 2 teams." } 
            }
        }
    }
}






