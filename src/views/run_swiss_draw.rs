use crate::Route;
use dioxus::prelude::*;
use swissdraw::{SwissDraw,Team,Game,load_draw_from_db};
use crate::DB;
use rusqlite::{Connection, Result};
use std::collections::HashMap;


const BLOG_CSS: Asset = asset!("/assets/styling/blog.css");

#[component]
pub fn Enter_Scores(sd_id: i64) -> Element {
    let conn = DB.lock().unwrap();
    let mut swiss_draw = load_draw_from_db(sd_id, &conn).expect("Failed to load Swiss Draw");
    // conn.close().expect("Failed to close DB connection");
    let round = swiss_draw.round;


    // Extract only the current round's games
    let games = swiss_draw.games.iter_mut()
        .filter(|game| game.round == round)
        .collect::<Vec<_>>();

    // Clone data for rendering without mutability
    let games_data = games.iter().map(|game| {
        (
            game.id.clone(),
            game.team_a.clone(),
            game.team_b.clone(),
            game.team_a_score,
            game.team_b_score,
        )
    }).collect::<Vec<_>>();

    let mut values = use_signal(HashMap::new);
    let mut submitted_values = use_signal(HashMap::new);
    let mut all_games_played = use_signal(|| false);


    rsx! {
        document::Link { rel: "stylesheet", href: BLOG_CSS }
        div {
            h1 { "Enter Scores for Round {round}" }

                if !submitted_values.read().is_empty() {
                    h2 { "Submitted! ✅" }
                }


                form {
                    oninput: move |ev| {
                        values.set(ev.values());
                    },
                    onsubmit: move |ev| {
                        ev.prevent_default(); // Prevent page reload
                        let submitted = ev.values();
                        submitted_values.set(submitted.clone());

                        // Update the scores in the SwissDraw struct
                        for (id, _, _, _, _) in games_data.iter() {
                            let team_a_key = format!("{id}:team_a");
                            let team_b_key = format!("{id}:team_b");

                            // Parse the scores from the submitted data
             
                            let a_score = submitted.get(&team_a_key).unwrap().as_value().parse::<i64>().unwrap();
                            let b_score = submitted.get(&team_b_key).unwrap().as_value().parse::<i64>().unwrap();

                            println!("Updating game {id}: {a_score} : {b_score}");

                            swiss_draw.edit_game_scores(
                                id.clone(),
                                a_score,
                                b_score,
                            );
                            }

                        // Save the updated SwissDraw back to the database
                        let conn = DB.lock().unwrap();
                        if let Err(e) = swiss_draw.sync_draw(&conn) {
                            println!("Failed to sync Swiss Draw: {e}");
                        }

                        // all_games_played.set(true);
                        if swiss_draw.check_games_played() {
                            all_games_played.set(true);
                        } else {
                            all_games_played.set(false);
                        }
                    },

                    for (i, (id, team_a, team_b, team_a_score, team_b_score)) in games_data.iter().enumerate() {
                        div {
                            "{team_a} vs {team_b}: "
                            input {
                                r#type: "number",
                                name: "{id}:team_a",
                                value: "{team_a_score}",
                            }
                            " : "
                            input {
                                r#type: "number",
                                name: "{id}:team_b",
                                value: "{team_b_score}",
                            }
                        }
                    }
                    button { r#type: "submit", value: "Submit", "Submit Scores" },
                }

            if *all_games_played.read() == true {
                h2 { "All Games Played!" }
                button {
                    onclick: move |_| {
                        use_navigator().replace(Route::Score_Draw { sd_id });
                    },
                    "Score Draw"
                }


            }
        }
    }
}


#[component]
pub fn Score_Draw(sd_id: i64) -> Element {
    let conn = DB.lock().unwrap();
    let mut swiss_draw = load_draw_from_db(sd_id, &conn).expect("Failed to load Swiss Draw");
    // conn.close().expect("Failed to close DB connection");
    let sd_id = swiss_draw.id;

    // If round is 0, just score the draw using run_draw
    if swiss_draw.round == 0 {
        swiss_draw.run_draw();
        // Sync the draw after running it
        // let conn = DB.lock().unwrap();
        swiss_draw.sync_draw(&conn).expect("Failed to sync Swiss Draw");
        // conn.close().expect("Failed to close DB connection");
        rsx! {
            document::Link { rel: "stylesheet", href: BLOG_CSS }
            div {
                id: "Score Draw",
                h1 { "Score Draw #{sd_id}!" }
                p { "Draw has been scored. (Round 0)" }
            }
            // add a button to take the user to the Enter_Scores page
            button {
                onclick: move |_| {
                    use_navigator().replace(Route::Enter_Scores { sd_id: sd_id });
                },
                "Go to Enter Scores"
            }

        }
    } else {
        // Print latest round's games
        let round = swiss_draw.round;
        let games = swiss_draw.games.iter()
            .filter(|game| game.round == round)
            .collect::<Vec<&Game>>();

        rsx! {
            document::Link { rel: "stylesheet", href: BLOG_CSS }
            div {
                id: "Score Draw",
                h1 { "Score Draw #{sd_id}!" }
                h2 { "Round {round} Games" }
                ul {
                    for game in games.iter() {
                        li { "{game.team_a} vs {game.team_b} - Score: {game.team_a_score} : {game.team_b_score}" }
                    }
                }
                p { "Please check the scores are accurate." }

                button {
                    onclick: move |_| {
                        use_navigator().replace(Route::Enter_Scores { sd_id: sd_id });
                        println!("Navigating Back to Enter Scores page for Draw #{sd_id}");
                    },
                    "Back To Enter Scores"
                }

                button {
                    onclick: move |_| {
                        // Spawn a task to run the scoring and DB update asynchronously
                        spawn({
                            let sd_id = sd_id;
                            let mut sd2 = swiss_draw.clone();
                            async move {
                                sd2.run_draw();
                                println!("there are {} games in this draw", sd2.games.len());
                                println!("Running async process");
                                let conn = DB.lock().unwrap();
                                if let Err(e) = sd2.sync_draw(&conn) {
                                    println!("Failed to sync Swiss Draw: {e}");
                                }
                                // After DB update, navigate to Enter_Scores
                                use_navigator().replace(Route::Enter_Scores { sd_id });
                                println!("Draw scored successfully! -> moving to Enter Scores page for Draw #{sd_id}");
                            }
                        });
                    },
                    "Run Scoring Process"
                }
            }
        }
    }
}
