use dioxus::prelude::*;
use rfd::FileDialog;
use csv::Reader;
use rusqlite::{Connection, Result};
use nalgebra::*;
use crate::DB;


fn csv_to_db(path: String, conn: &Connection) ->  rusqlite::Result<()> {
    let mut reader = Reader::from_path(path).expect("Failed to read CSV file");
    // let mut stmt = conn.prepare("CREATE TABLE if not exists temp_results (round REAL, teamA TEXT, teamB TEXT, teamAscore REAL, teamBscore REAL)")?;
    let mut stmt = conn.prepare("CREATE TABLE if not exists temp_results (round REAL, teamA TEXT, teamB TEXT, teamAscore REAL, teamBscore REAL)")?;
    stmt.execute([])?;

    for result in reader.records() {
        let record = result.expect("Failed to read record");
        let round: String = record[0].parse().unwrap();
        let team_a: String = record[1].to_string();
        let team_b: String = record[2].to_string();
        let team_a_score: i64 = record[3].parse().unwrap();
        let team_b_score: i64 = record[4].parse().unwrap();

        conn.execute(
            "INSERT INTO temp_results (round, teamA, teamB, teamAscore, teamBscore) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![round, team_a, team_b, team_a_score, team_b_score],
        )?;
    }
    Ok(())
}

fn print_matrix_by_rows(matrix: &DMatrix<f64>) {
    for i in 0..matrix.nrows() {
        let row = matrix.row(i);
        for val in row.iter() {
            print!("{:.2} ", val);
        }
        println!();
    }
}

fn get_string_vec(query_string: String, conn: &Connection) -> Result<DVector<String>> {
    let mut stmt = conn.prepare(&query_string)?;
    let list_vals: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();
    Ok(DVector::from_vec(list_vals))
}

fn get_vec(query_string: String, conn: &Connection) -> Result<DVector<f64>> {
    let mut stmt = conn.prepare(&query_string)?;
    let list_vals: Vec<f64> = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect();
    Ok(DVector::from_vec(list_vals))
}

fn games_matrix(team_list: &DVector<String>, teams_played: &DVector<String>) -> DMatrix<f64> {
    let mut matrix = DMatrix::zeros(teams_played.len(), team_list.len());

    for (j, team_name) in team_list.iter().enumerate() {
        for (i, opponent_name) in teams_played.iter().enumerate() {
            if team_name == opponent_name {
                matrix[(i, j)] = 1.0;
            }
        }
    }
    matrix
}

// this does the strength calculations on a bunch of games.
// it takes a path to a csv file, and then does the calculations and returns the scores and the team
fn calculate_strengths(path: String) -> Result<(DVector<String>, DVector<f64>)> {
    // let conn = Connection::open_in_memory()?;
    let conn = DB.lock().unwrap();

    csv_to_db(path, &conn).expect("Failed to load CSV into DB");
    // let querystring = &format!(r#"CREATE TABLE temp_results AS SELECT * FROM read_csv_auto('{path}');"#,path = path).to_string();
    // conn.execute(querystring,[],)?;

    let team_list = get_string_vec("with dist as (SELECT DISTINCT teamA as team FROM temp_results UNION all SELECT DISTINCT teamB as team FROM temp_results) select distinct team from dist order by team".to_string(), &conn)?;
    let team_a_vec = get_string_vec("SELECT teamA FROM temp_results".to_string(), &conn)?;
    let team_b_vec = get_string_vec("SELECT teamB FROM temp_results".to_string(), &conn)?;
    let margin_vec = get_vec("SELECT teamAscore - teamBscore FROM temp_results".to_string(), &conn)?;

    conn.execute("drop TABLE if exists temp_results",[]).expect("Failed to drop temp_results table");
    

    let m_a = games_matrix(&team_list, &team_a_vec);
    let m_b = games_matrix(&team_list, &team_b_vec);
    let m = m_a - m_b; // this is the matrix of games played, with 1s and -1s

    // print_matrix_by_rows(&m);

    let pinv: DMatrix<f64> = m.pseudo_inverse(1e-10).expect("PINV failed");
    let scores = pinv * margin_vec;

    Ok((team_list, scores))
}


const ECHO_CSS: Asset = asset!("/assets/styling/echo.css");

/// Echo component that demonstrates fullstack server functions.
#[component]
pub fn Load_Results() -> Element {
    // use_signal is a hook. Hooks in dioxus must be run in a consistent order every time the component is rendered.
    // That means they can't be run inside other hooks, async blocks, if statements, or loops.
    //
    // use_signal is a hook that creates a state for the component. It takes a closure that returns the initial value of the state.
    // The state is automatically tracked and will rerun any other hooks or components that read it whenever it changes.
    let mut response = use_signal(|| String::new());

    rsx! (
        div { "File dialog" }
        // Pick file button
        button {
            onclick: move |_| {
                let path = FileDialog::new()
                .set_directory("/")
                .pick_file()
                .and_then(|path| path.to_str().map(|s| s.to_string()));

                println!("picked file: {:?}", path);

                // let path_str = path.expect("REASON").to_str();

                let (team_list, scores) = calculate_strengths(path.unwrap()).unwrap();
                // I want to print the team list and scores next to each other
            
                for (team, score) in team_list.iter().zip(scores.iter()) {
                    println!("{}: {:.2}", team, score);
                }

            },
            "Pick file"
            }

       )
}


// When the server function is called from the client, it will just serialize the arguments, call the API, and deserialize the
// response.
#[server]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    // The body of server function like this comment are only included on the server. If you have any server-only logic like
    // database queries, you can put it here. Any imports for the server function should either be imported inside the function
    // or imported under a `#[cfg(feature = "server")]` block.
    Ok(input)
}
