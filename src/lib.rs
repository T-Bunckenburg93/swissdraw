// pub mod components;
// use rand::prelude::*;
use nalgebra::*;
use good_lp::*;
use std::collections::HashMap;
use csv::Reader;
use serde::Deserialize;
use rusqlite::{Connection, Result, params};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashSet;

// pub enum ByeLocation {
//     Start,
//     Middle,
//     End,
// }

// define my custom structs
#[derive(Clone)]
#[derive(Debug)]
pub struct Game {
    // pub sd_id: u64, // random id to keep SD's unique, do we need here?
    pub id: i64, // random id to keep games unique
    pub round: i64,
    pub field: i64,
    pub team_a: String,
    pub team_b: String,
    pub team_a_score: i64,
    pub team_b_score: i64,
    pub streamed: bool,
    pub played: bool,
}

// this is for deserializing from a csv file
#[derive(Debug, Deserialize)]
struct GameRow {
    round: Option<i64>,
    teamA: String,
    teamB: String,
    teamAScore: i64,
    teamBScore: i64,
    field: String,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Team {
    pub id: i64,
    pub name: String,
    pub rank: f64,
}
#[derive(Clone, Debug)]
pub struct SwissDraw{
    pub id : i64,
    pub name: String,
    pub round: i64,
    pub team_list: Vec<Team>,
    pub latest_rank:Vec<Team>,
    pub games: Vec<Game>,
    // pub bye: ByeLocation,
}

// can I define a way to print them all nicely?
impl Game {
    pub fn new(field: i64, team_a: String, team_b: String) -> Game {
        Game {
            id: rand::random(),
            round:0 ,
            field,
            team_a,
            team_b,
            team_a_score: 0,
            team_b_score: 0,
            streamed: false,
            played: false,
        }
    }
}

impl Team {
    pub fn new(id: i64, name: String, rank: f64) -> Team {
        Team {
            id,
            name,
            rank,
        }
    }
}

impl SwissDraw {
    // pub fn new(round: u64, bye: ByeLocation) -> SwissDraw {
    pub fn new() -> SwissDraw {
        SwissDraw {
            id: rand::random(),
            name: "".to_string(),
            round: 0,
            team_list: Vec::new(),
            latest_rank: Vec::new(),
            games: Vec::new(),
            // bye,
        }
    }

    pub fn add_game(&mut self, game: Game) {
        self.games.push(game);
    }

    pub fn get_games(&self) -> &Vec<Game> {
        &self.games
    }
    pub fn add_team(&mut self, name: String, rank: f64) {
        let team = Team::new(rand::random(), name, rank);
        self.team_list.push(team);
    }

    pub fn add_teams(&mut self, teams: Vec<Team>) {
        for team in teams {
            self.team_list.push(team);
        }
    }

    pub fn edit_game_scores(&mut self, id: i64, team_a_score: i64, team_b_score: i64) {
        for game in &mut self.games {
            if game.id == id {
                game.team_a_score = team_a_score;
                game.team_b_score = team_b_score;
                game.played = true;
            }
        }
    }

    pub fn check_games_played(&self) -> bool {
        for i in &self.games {
            if !i.played {
                println!("Game {} vs {} has not been played", i.team_a, i.team_b);
                return false;
            }
        }
        println!("All games have been played");
        true
    }


    pub fn get_strengths(&mut self) -> Vec<Team> {
        let mut strengths = self.team_list.clone();
        if self.round == 0 {
            // if round is 0, sort by initial rank, high to low
            strengths.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
            self.latest_rank = strengths.clone();
        } else {
            // get the strengths from prior games
            strengths = calculate_strengths(self.games.clone(), &self.team_list);
  
            // sort by score
            strengths.sort_by(|a, b| b.rank.partial_cmp(&a.rank).unwrap());
            self.latest_rank = strengths.clone();

            println!("Strengths: {:?}", strengths);
        }
        strengths
    }


    pub fn get_cost_matrix(&mut self) -> DMatrix<f64> {
        // get the cost matrix based on the team strengths
        let strengths = self.get_strengths();
        let cost_matchup_m = cost_matchup(&strengths);
        let cost_self_m = cost_self(&strengths);
        let cost_prev_m = cost_prev_games(&strengths, &self.games);

        // add elements of the two matrices together
        let cost_matrix_m = cost_matchup_m + cost_self_m + cost_prev_m;
        cost_matrix_m
    }

    pub fn run_draw(&mut self) {
        // check if the draw is valid
        if self.check_draw().is_err() {
            println!("Draw is not valid");
            return;
        }
        // get the cost matrix
        let strengths = self.get_strengths();
        let cost_matrix = self.get_cost_matrix();

        // run the optimization
        let games = opt(&strengths, &cost_matrix);

        self.round += 1;
        println!("Running Swiss Draw for round {}", self.round);



        // print the games
        for game in &games {
            println!("Game: {} vs {}", game.team_a, game.team_b);
        }
        // add the games to the swiss draw
        let mut field_n = 1;

        for mut game in games {
            game.round = self.round;
            if  game.team_a == "_BYE_" || game.team_b == "_BYE_" {
                game.team_a_score = 0;
                game.team_b_score = 0;
                game.played = true;
                game.streamed = false;
                game.field = 0;
            }


            // print a all the game fields
            // println!("Game: {} vs {} field: {}", game.team_a, game.team_b, game.field);

            // set the field number
            game.field = field_n;
           
            self.add_game(game);
            field_n += 1;
        }
        // println!("There are {} games", self.games.len());
    }

    // here are a bunch of programs to load in from a CSV

    pub fn csv_to_games(&mut self, path: String) {
        let mut rdr = Reader::from_path(path).expect("Failed to open CSV file");

        let mut id_counter = self.games.len() as i64 + 1;  // continue id from existing games

        for result in rdr.deserialize() {
            let row: GameRow = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("CSV parse error: {}", e);
                    continue;  // skip bad rows
                }
            };
            let game = Game {
                id: id_counter,
                round: row.round.unwrap_or(1),
                field: 1,
                team_a: row.teamA,
                team_b: row.teamB,
                team_a_score: row.teamAScore,
                team_b_score: row.teamBScore,
                streamed: false,
                played: true,
            };
            self.games.push(game);
            id_counter += 1;
        }
    }

    // checks if the draw is valid and adds a bye team if needed
    fn check_draw(&mut self) -> Result<bool, String> {

        // check if there are an odd number of teams
        // if so, add a bye team with name _BYE_ and id 0
        if self.team_list.len() % 2 == 1 {
            let bye_team = Team::new(0, "_BYE_".to_string(), 0.0);
            self.team_list.push(bye_team);
        }

        // check if the draw is valid
        // check if there are any games that have been played
        for game in &self.games {
            if !game.played {
                println!("Game {} vs {} has not been played", game.team_a, game.team_b);
                return Err("Not all games have been played".into());
            }
        }

        Ok(true)
    }

    

    // fn to save the draw to a db.
    pub fn sync_draw(&self, conn: &Connection) -> Result<()> {
        // ok, so first up. Look at the draw table and see if it exists
        let sd_id = self.id;

        let sd_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM draw WHERE id = ?1)",
            params![&sd_id],
            |row| row.get(0),
        ).expect("Failed to query draw table");

        
        if !sd_exists {
            println!("Swiss draw does not exist, creating new one");
            save_draw(self, conn)?;
        }
        else {
            println!("Swiss draw exists so we're updating it");
            // update the draw
            update_draw(self, conn)?;
        }
        Ok(())

    }


}

// this will load the draw from the db
// the teams and games will be loaded into the swissdraw struct
// they exist in their own tables and are linked by the sd_id
// only load games that are not deleted
pub fn load_draw_from_db(sd_id: i64, conn: &Connection) -> Result<SwissDraw> {
    // Load draw info
    let mut stmt = conn.prepare("SELECT id, name FROM draw WHERE id = ?1")?;
    let draw_row = stmt.query_row(rusqlite::params![sd_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
        ))
    })?;

    let (id, name) = draw_row;

    // Load teams
    let mut team_list = Vec::new();
    let mut stmt = conn.prepare("SELECT id, name, rank FROM teams WHERE sd_id = ?1")?;
    let team_iter = stmt.query_map(rusqlite::params![sd_id], |row| {
        Ok(Team {
            id: row.get(0)?,
            name: row.get(1)?,
            rank: row.get(2)?,
        })
    })?;

    for team in team_iter {
        team_list.push(team?);
    }

    // Load games (not deleted)
    let mut games = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT id, round, field, team_a_id, team_b_id, team_a_score, team_b_score, streamed, played
         FROM games WHERE sd_id = ?1 AND _meta__is_deleted = 0"
    )?;
    let game_iter = stmt.query_map(rusqlite::params![sd_id], |row| {
        Ok(Game {
            id: row.get(0)?,
            round: row.get(1)?,
            field: row.get(2)?,
            team_a: row.get(3)?,
            team_b: row.get(4)?,
            team_a_score: row.get(5)?,
            team_b_score: row.get(6)?,
            streamed: row.get(7)?,
            played: row.get(8)?,
        })
    })?;

    let mut max_round = 0;
    for game in game_iter {
        let g = game?;
        if g.round > max_round {
            max_round = g.round;
        }
        games.push(g);
    }

    Ok(SwissDraw {
        id,
        name,
        round: max_round,
        team_list,
        latest_rank: Vec::new(),
        games,
    })
}









// ok, so I'm going to produce a bunch of team_N x team_N sized matrices
// and then use the matrix to determine the best matchups. order is important and will be the from the teamVector from get strengths.
fn cost_matchup(team_list: &Vec<Team>) -> DMatrix<f64> {

    let n = team_list.len();
    let mut cost_matrix = DMatrix::zeros(n, n);

    for i in 0..n {
        for j in 0..n {
            if i != j {
                // calculate the cost based on the difference in rank
                let cost = (team_list[i].rank - team_list[j].rank).abs() as f64;
                cost_matrix[(i, j)] = cost;
            }
        }
    }
    cost_matrix
}

fn cost_self(team_list: &Vec<Team>) -> DMatrix<f64> {
    let n = team_list.len();
    let mut cost_matrix = DMatrix::zeros(n, n);

    for i in 0..n {
        for j in 0..n {
            if i == j {
                // calculate the cost based on the difference in rank
                // let cost = (team_list[i].rank - team_list[j].rank).abs() as f64;
                cost_matrix[(i, j)] = 1000000.0; // high cost for self-matchup
            }
        }
    }
    cost_matrix
}

fn cost_prev_games(team_list: &Vec<Team>, game_list: &Vec<Game>) -> DMatrix<f64> {
    let n = team_list.len();
    let mut cost_matrix = DMatrix::zeros(n, n);

    // turn the team_list into a hashmap for quick lookup
    let mut team_ind = HashMap::new();
    for (i, team) in team_list.iter().enumerate() {
        team_ind.insert(team.name.clone(), i);
    }

    // loop over the games and add the prior matchups
    for game in game_list {

        println!("Game: {} vs {}", game.team_a, game.team_b);
        let team_a_ind = team_ind.get(&game.team_a).unwrap();
        let team_b_ind = team_ind.get(&game.team_b).unwrap();

        // calculate the cost based on the difference in rank
        let cost = 100.0; // high cost for self-matchup
        cost_matrix[(*team_a_ind, *team_b_ind)] += &cost;
        cost_matrix[(*team_b_ind, *team_a_ind)] += &cost;
    }
    cost_matrix
}



// This function takes a vector of games and calculates the strengths of the teams
// Kinda like a logistic reression. 
// bye games are not included for the calculation, but come back as a team with 0 strength afterwards
fn calculate_strengths(games: Vec<Game>, teams: &Vec<Team>) -> Vec<Team>  {

    // I want the teams from team as a string vector
    let mut team_list = Vec::new();
    for team in teams {
        team_list.push(team.name.clone());
    }

    let mut bye = false;

    // if there are any games with bye, remove them
    let game_list: Vec<Game> = games.into_iter().filter(|game| game.team_a != "_BYE_" && game.team_b != "_BYE_").collect();

    // check if there are an odd number of teams
    if &team_list.len() % 2 == 1 {
        // if there are an odd number of teams, add a bye team (at the end)
        bye = true;
    }


    let team_sz = team_list.len();
    let game_sz = game_list.len();

    // create a matrix of where the teams have played
    let mut m_a = DMatrix::zeros(game_sz, team_sz);
    let mut m_b = DMatrix::zeros(game_sz, team_sz);

    for (j, team_name) in team_list.iter().enumerate() {
        for (i, game) in game_list.iter().enumerate() {
            if *team_name == game.team_a {
                m_a[(i, j)] = 1.0;
            }
            if *team_name == game.team_b {
                m_b[(i, j)] = 1.0;
            }
        }
    }

    let m = m_a - m_b;

    // create the margin vector. 
    let mut margin_v = DVector::zeros(game_sz);
    for (i, game) in game_list.iter().enumerate() {
        margin_v[i] = (game.team_a_score as f64) - (game.team_b_score as f64);
    }

    // println!("m_a: {:?}", m);

    let pinv: DMatrix<f64> = m.pseudo_inverse(1e-10).expect("PINV failed");
    let scores = pinv * margin_v;

    // ok, so now I have the scores for each team
    // lets create a new team list with the scores
    let mut team_scores = Vec::new();
    for (i, team_name) in team_list.iter().enumerate() {
        let team = Team::new(teams[i].id, team_name.clone(), scores[i] as f64);
        team_scores.push(team);
    }

    if bye {
        // add the bye team to the list
        let bye_team = Team::new(0, "_BYE_".to_string(), 0.0);
        team_scores.push(bye_team);
    }

    team_scores

}

// fn matchup_cost(matchup: Variable, cost: <f64>) -> Expression {
//     (matchup * cost).into()
// }

pub fn opt(teams:  &Vec<Team>, costs: &DMatrix<f64>) -> Vec<Game> {
    
    let team_sz = teams.len();
    let mut problem = ProblemVariables::new();

    // create the variables in a 2D vector
    let vars: Vec<Vec<Variable>> = (0..team_sz)
    .map(|_| {
        (0..team_sz)
            .map(|_| problem.add(variable().binary()))
            .collect()
    })
    .collect();

    // create the expression dynamically
    let mut objective = Expression::from(0.0);
    for i in 0..costs.nrows() {
        for j in 0..costs.ncols() {
            objective += costs[(i, j)] * vars[i][j];
        }
    }

    let mut constraints = Vec::new();

    for i in 0..costs.nrows() {
        let mut row_expr = Expression::from(0.0);
        for j in 0..costs.ncols() {
            row_expr += vars[i][j];
        }
        constraints.push(row_expr.eq(1));
    }
        
    // let constraint_sz = &constraints.len();
    // println!("constraints size: {}", constraint_sz);

    for i in 0..costs.ncols() {
        let mut row_expr = Expression::from(0.0);
        for j in 0..costs.nrows() {
            row_expr += vars[i][j];
        }
        constraints.push(row_expr.eq(1));
    }

    // let constraint_sz = &constraints.len();
    // println!("constraints size: {}", constraint_sz);

    // ok so the final constraint is that vars[i][j] == vars[j][i]
    for i in 0..costs.nrows() {
        for j in (i)..costs.ncols() {
        // for j in 0..costs.ncols() { // this does all equality constraints. Might not be needed?
            constraints.push((vars[i][j] - vars[j][i]).eq(0));
        }
    }

    let problem_sz = vars.len();
    println!("Problem size: {}", problem_sz);

    let constraint_sz = &constraints.len();
    println!("constraints size: {}", constraint_sz);


    // Build model and apply all constraints
    let mut model = problem.minimise(&objective).using(default_solver);
    for c in constraints {
        model = model.with(c);
    }

    let solution = model.solve().unwrap();
    // for i in 0..costs.nrows() {
    //     for j in 0..costs.ncols() {
    //         println!("x[{}][{}] = {}", i, j, solution.value(vars[i][j]));
    //     }
    // }
    println!("Solution: {:?}", solution.eval(&objective));

    // ok, so now I want to get the teams that are matched up as a vec<Game>
    // I dont know what the round or field are yet, so I don't want them set
    let mut games = Vec::new();
    for i in 0..costs.nrows() {
        for j in (i+1)..costs.ncols() {
            if solution.value(vars[i][j]) == 1.0 {
                let game = Game::new( 0, teams[i].name.clone(), teams[j].name.clone());
                println!("Gameid: {}, {} vs {}",game.id, game.team_a, game.team_b);
                games.push(game);
            }
        }
    }

    games

}


// fn to save a new draw to a db.
fn save_draw(swissdraw: &SwissDraw, conn: &Connection) -> Result<()> {

    println!("Saving swiss draw to db");
let now = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs())
    .expect("Timestamp too large for i64");
    // I want to create a new row in the draw table
    conn.execute(
        "INSERT INTO draw (id, name, last_modified) VALUES (?1, ?2, ?3)",
        rusqlite::params![swissdraw.id, swissdraw.name, &now ],
    ).unwrap_or_else(|e| { panic!("Failed to insert draw into db: {}", e);});

    println!("saved draw to db");


    // now we want to add the teams to the teams table
    for team in &swissdraw.team_list {
        // println!("{} {} {} {}",swissdraw.id, team.id, team.name, team.rank);
        conn.execute(
            "INSERT INTO teams (sd_id, id, name, rank) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![swissdraw.id, team.id, team.name, team.rank],
        ).unwrap_or_else(|e| { panic!("Failed to insert team into db: {}", e);});
    };
    println!("saved teams to db");

    // and the games to the games table
    for game in &swissdraw.games {
        conn.execute(
            "INSERT INTO games (sd_id, id, round, field, team_a_id, team_b_id, team_a_score, team_b_score, played, streamed, _meta__is_current, _meta__is_deleted, _meta__last_modified) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![swissdraw.id, game.id, game.round, game.field, game.team_a, game.team_b, game.team_a_score, game.team_b_score, game.played, game.streamed, true, false, &now],
        ).unwrap_or_else(|e| { panic!("Failed to insert game into db: {}", e);});
    };
    println!("saved games to db");
    Ok(())
}

// this will either update draw values IF they have changed.
// or create new ones if they don't exist in the db
// or set the db games to deleted if they are not in the current draw but in the db.
fn update_draw(swissdraw: &SwissDraw, conn: &Connection) -> Result<()> {
    let now = i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs())
        .expect("Timestamp too large for i64");

    // Update draw info
    conn.execute(
        "UPDATE draw SET name = ?1, last_modified = ?2 WHERE id = ?3",
        rusqlite::params![swissdraw.name, &now, swissdraw.id],
    )?;

    // Update or insert teams
    for team in &swissdraw.team_list {
        let updated = conn.execute(
            "UPDATE teams SET name = ?1, rank = ?2 WHERE sd_id = ?3 AND id = ?4",
            rusqlite::params![team.name, team.rank, swissdraw.id, team.id],
        )?;
        if updated == 0 {
            conn.execute(
                "INSERT INTO teams (sd_id, id, name, rank) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![swissdraw.id, team.id, team.name, team.rank],
            )?;
        }
    }
    // Update or insert games
    let mut current_game_ids = HashSet::new();
    println!("n games to add/update {}", swissdraw.games.len());
    for game in &swissdraw.games {

        // println!("Updating game: {} vs {}", game.team_a, game.team_b);
        current_game_ids.insert(game.id);
        let updated = conn.execute(
            "UPDATE games SET round = ?1, field = ?2, team_a_id = ?3, team_b_id = ?4, team_a_score = ?5, team_b_score = ?6, played = ?7, streamed = ?8, _meta__is_current = 1, _meta__is_deleted = 0, _meta__last_modified = ?9 WHERE sd_id = ?10 AND id = ?11",
            rusqlite::params![
                game.round,
                game.field,
                game.team_a,
                game.team_b,
                game.team_a_score,
                game.team_b_score,
                game.played,
                game.streamed,
                &now,
                swissdraw.id,
                game.id
            ],
        )?;
        println!("Updated game {}: {} vs {}", game.id, game.team_a, game.team_b);
        if updated == 0 {
            conn.execute(
                "INSERT INTO games (sd_id, id, round, field, team_a_id, team_b_id, team_a_score, team_b_score, played, streamed, _meta__is_current, _meta__is_deleted, _meta__last_modified) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 1, 0, ?11)",
                rusqlite::params![
                    swissdraw.id,
                    game.id,
                    game.round,
                    game.field,
                    game.team_a,
                    game.team_b,
                    game.team_a_score,
                    game.team_b_score,
                    game.played,
                    game.streamed,
                    &now
                ],
            )?;
            println!("Inserted new game id {}: {} vs {}",game.id, game.team_a, game.team_b);
        }
    }

    // Mark games as deleted if they are in the db but not in the current draw
    let mut stmt = conn.prepare("SELECT id FROM games WHERE sd_id = ?1 AND _meta__is_deleted = 0")?;
    let db_game_ids = stmt.query_map(rusqlite::params![swissdraw.id], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect::<HashSet<i64>>();

    for db_id in db_game_ids.difference(&current_game_ids) {
        conn.execute(
            "UPDATE games SET _meta__is_deleted = 1, _meta__is_current = 0, _meta__last_modified = ?1 WHERE sd_id = ?2 AND id = ?3",
            rusqlite::params![&now, swissdraw.id, db_id],
        )?;
    }

    Ok(())
}



