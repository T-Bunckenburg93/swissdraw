use dioxus::prelude::*;
use rfd::FileDialog;
use csv::Reader;
use rusqlite::{Connection, Result};
use nalgebra::*;
use crate::DB;
use swissdraw::{SwissDraw,Team,Game};
use itertools::Itertools;


const LR_CSS: Asset = asset!("/assets/styling/load_results.css");

#[derive(Debug)]
struct TeamScore {
    name: String,
    score: f64,
}


#[component]
pub fn Score_Prior() -> Element {
    // Use SwissDraw as the signal state
   let mut sd_init = SwissDraw::new();
   sd_init.round = 1;
   let mut swiss_draw = use_signal(|| sd_init);


    swiss_draw.write().round = 1;

    // Rendered teams and scores
    let swiss_draw_lock = swiss_draw.read();
    let teams = &swiss_draw_lock.latest_rank;
    let teams_rendered = if !teams.is_empty() {
        rsx! {
            table { class: "team-table",
                thead {
                    tr {
                        th { class: "header-cell", "Team" }
                        th { class: "header-cell", "Score" }
                    }
                }
                tbody {
                    {teams.iter().map(|team| {
                        if team.name != "_BYE_" {
                            let formatted_score = format!("{:.3}", team.rank);
                            rsx! {
                                tr {
                                    td { class: "data-cell", "{team.name}" }
                                    td { class: "data-cell", "{formatted_score}" }
                                }
                            }
                        } else { rsx! {} }
                    })}
                }
            }
        }
    } else {
        rsx! { p { 
            "Load in a bunch of game results from a CSV \n"
            "results should be in the format of teamA,teamB,teamAscore,teamBscore \n"
            "its assumed that all teams are from the same tournament"
        } }
    };

    rsx! (
        document::Link { rel: "stylesheet", href: LR_CSS }
        div { "File dialog" }
        // Pick file button
        button {
            onclick: move |_| {
                let path = FileDialog::new()
                .set_directory("/")
                .pick_file()
                .and_then(|path| path.to_str().map(|s| s.to_string()));

                println!("picked file: {:?}", path);

                // recreate the swissDraw object.

                let mut sd = swiss_draw.write();

                let mut new_sd = SwissDraw::new();
                new_sd.round = 1;
                
                *sd = new_sd;
                // Load games into SwissDraw
                sd.csv_to_games(path.unwrap());

                println!("Loaded games: {:?}", sd.games.len());

                for g in &sd.games {
                    println!("{:?}", g);
                }

                // I want to add all the teams to the teamlist.

                // I want to create an empty string vector


                let mut team_a_v = Vec::new();
                let mut team_b_v = Vec::new();
                for g in &sd.games {
                    team_a_v.push(g.team_a.clone());
                    team_b_v.push(g.team_b.clone());
                }
                team_a_v.append(&mut team_b_v);
                let team_v = team_a_v.into_iter().unique();

                for i in team_v {
                    sd.add_team(i,1.0);
                }


                let strength = sd.get_strengths();
                sd.latest_rank = strength.clone();

                println!("Loaded strengths: {:?}", strength.len());


                // Print teams and scores
                for t in &sd.latest_rank {
                    println!("{:20} | {:.2}", t.name, t.rank);
                }
            },
            "Pick file"
        }
        {teams_rendered}
    )
}



