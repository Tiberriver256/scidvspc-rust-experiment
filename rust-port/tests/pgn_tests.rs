//! Integration tests using fixtures generated from C++

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use scid_pgn::{Game, PgnWriter};
use scid_pgn::pgn::{PgnOptions, PgnFormat};

#[derive(Debug, Deserialize, Serialize)]
struct TestFixture {
    name: String,
    description: String,
    input: GameInput,
    expected_pgn: String,
    options: FixtureOptions,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameInput {
    event: String,
    site: String,
    date: String,
    round: String,
    white: String,
    black: String,
    result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    white_elo: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    black_elo: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eco: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct FixtureOptions {
    format: String,
    include_comments: bool,
    include_variations: bool,
    include_nags: bool,
    short_header: bool,
}

fn load_fixture(path: &Path) -> TestFixture {
    let content = fs::read_to_string(path)
        .expect(&format!("Failed to read fixture: {}", path.display()));
    serde_json::from_str(&content)
        .expect(&format!("Failed to parse fixture: {}", path.display()))
}

fn parse_result(result_str: &str) -> scid_pgn::types::GameResult {
    match result_str {
        "1-0" => scid_pgn::types::GameResult::White,
        "0-1" => scid_pgn::types::GameResult::Black,
        "1/2-1/2" => scid_pgn::types::GameResult::Draw,
        _ => scid_pgn::types::GameResult::Unknown,
    }
}

fn parse_format(format_str: &str) -> PgnFormat {
    match format_str {
        "html" => PgnFormat::Html,
        "latex" => PgnFormat::Latex,
        "color" => PgnFormat::Color,
        _ => PgnFormat::Plain,
    }
}

fn fixture_to_game(input: &GameInput) -> Game {
    let mut game = Game::new();
    game.event = input.event.clone();
    game.site = input.site.clone();
    game.date = input.date.clone();
    game.round = input.round.clone();
    game.white = input.white.clone();
    game.black = input.black.clone();
    game.result = parse_result(&input.result);
    game.white_elo = input.white_elo;
    game.black_elo = input.black_elo;
    game.eco = input.eco.clone();
    game
}

fn normalize_pgn(pgn: &str) -> String {
    // Normalize whitespace and line endings for comparison
    pgn.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_fixtures() {
    // This test will run once fixtures are generated
    // For now, we'll just verify the infrastructure works
    
    let fixtures_dir = Path::new("tests/fixtures");
    if !fixtures_dir.exists() {
        println!("Fixtures directory not found. Run generate_test_fixtures.sh first.");
        return;
    }
    
    // Find all JSON fixtures
    let fixture_files: Vec<_> = glob::glob("tests/fixtures/**/*.json")
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .collect();
    
    if fixture_files.is_empty() {
        println!("No fixtures found. Run generate_test_fixtures.sh first.");
        return;
    }
    
    let mut passed = 0;
    let mut failed = 0;
    
    for fixture_path in fixture_files {
        let fixture = load_fixture(&fixture_path);
        
        // Convert fixture to game
        let game = fixture_to_game(&fixture.input);
        
        // Create writer with appropriate options
        let options = PgnOptions {
            format: parse_format(&fixture.options.format),
            include_comments: fixture.options.include_comments,
            include_variations: fixture.options.include_variations,
            include_nags: fixture.options.include_nags,
            short_header: fixture.options.short_header,
        };
        
        let writer = PgnWriter::new(options);
        let actual_pgn = writer.write(&game).expect("Failed to write PGN");
        
        // Normalize both for comparison
        let expected_normalized = normalize_pgn(&fixture.expected_pgn);
        let actual_normalized = normalize_pgn(&actual_pgn);
        
        if expected_normalized == actual_normalized {
            passed += 1;
        } else {
            failed += 1;
            println!("\n❌ FAILED: {}", fixture.name);
            println!("  Description: {}", fixture.description);
            println!("  Expected:\n{}", fixture.expected_pgn);
            println!("  Actual:\n{}", actual_pgn);
        }
    }
    
    println!("\n=== Test Summary ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);
    
    if failed > 0 {
        panic!("{} test(s) failed", failed);
    }
}

#[test]
fn test_manual_simple_game() {
    // A manual test that doesn't require fixtures
    let mut game = Game::new();
    game.event = "Test Event".to_string();
    game.site = "Test Site".to_string();
    game.date = "2024.02.04".to_string();
    game.round = "1".to_string();
    game.white = "Player, White".to_string();
    game.black = "Player, Black".to_string();
    game.result = scid_pgn::types::GameResult::White;
    game.white_elo = Some(2500);
    
    let writer = PgnWriter::default();
    let pgn = writer.write(&game).unwrap();
    
    // Basic assertions
    assert!(pgn.contains("[Event \"Test Event\"]"));
    assert!(pgn.contains("[White \"Player, White\"]"));
    assert!(pgn.contains("[WhiteElo \"2500\"]"));
    assert!(pgn.contains("1-0"));
}
