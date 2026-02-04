/*
 * Test Fixture Generator for SCID PGN Conversion
 * 
 * This tool reads SCID games and outputs test fixtures in JSON format
 * that can be used to validate both C++ and Rust implementations.
 */

#include <iostream>
#include <fstream>
#include <string>
#include <vector>
#include <cstdlib>
#include <getopt.h>
#include "game.h"
#include "textbuf.h"
#include "json_writer.h"

enum FixtureType {
    SIMPLE,
    VARIATIONS,
    COMMENTS,
    NAGS,
    SPECIAL,
    FORMATS
};

struct Options {
    FixtureType type;
    int count;
    std::string output_dir;
};

void print_usage(const char* program_name) {
    std::cout << "Usage: " << program_name << " [OPTIONS]\n"
              << "\nOptions:\n"
              << "  --type TYPE        Fixture type: simple, variations, comments, nags, special, formats\n"
              << "  --count N          Number of fixtures to generate (default: 10)\n"
              << "  --output DIR       Output directory for fixtures\n"
              << "  --help             Show this help message\n"
              << std::endl;
}

FixtureType parse_type(const std::string& type_str) {
    if (type_str == "simple") return SIMPLE;
    if (type_str == "variations") return VARIATIONS;
    if (type_str == "comments") return COMMENTS;
    if (type_str == "nags") return NAGS;
    if (type_str == "special") return SPECIAL;
    if (type_str == "formats") return FORMATS;
    
    std::cerr << "Unknown fixture type: " << type_str << std::endl;
    exit(1);
}

void create_simple_game(Game* game, int index) {
    // Create a basic game with no variations or comments
    game->Clear();
    
    // Set tags
    char buffer[256];
    snprintf(buffer, sizeof(buffer), "Test Event %d", index);
    game->SetEventStr(buffer);
    
    snprintf(buffer, sizeof(buffer), "Test Site %d", index);
    game->SetSiteStr(buffer);
    
    game->SetWhiteStr("Player, White");
    game->SetBlackStr("Player, Black");
    game->SetRoundStr("1");
    game->SetDate(date_EncodeFromString("2024.02.04"));
    game->SetResult(RESULT_White);
    game->SetWhiteElo(2500);
    game->SetBlackElo(2480);
    
    // Add some moves (Scholar's Mate as a simple example)
    simpleMoveT sm;
    Position pos;
    pos.StdStart();
    
    // 1. e4
    pos.MakeMove(&sm, E2, E4);
    game->AddMove(&sm, "e4");
    
    // 1... e5
    pos.MakeMove(&sm, E7, E5);
    game->AddMove(&sm, "e5");
    
    // 2. Bc4
    pos.MakeMove(&sm, F1, C4);
    game->AddMove(&sm, "Bc4");
    
    // 2... Nc6
    pos.MakeMove(&sm, B8, C6);
    game->AddMove(&sm, "Nc6");
    
    // 3. Qh5
    pos.MakeMove(&sm, D1, H5);
    game->AddMove(&sm, "Qh5");
    
    // 3... Nf6
    pos.MakeMove(&sm, G8, F6);
    game->AddMove(&sm, "Nf6");
    
    // 4. Qxf7#
    pos.MakeMove(&sm, H5, F7);
    game->AddMove(&sm, "Qxf7#");
}

void create_game_with_variations(Game* game, int index) {
    create_simple_game(game, index);
    
    // Go back and add a variation
    game->MoveBackup();
    game->MoveBackup();
    game->AddVariation();
    
    // Add alternative move in variation
    simpleMoveT sm;
    game->CurrentPos->MakeMove(&sm, G8, H6);
    game->AddMove(&sm, "Nh6");
}

void create_game_with_comments(Game* game, int index) {
    create_simple_game(game, index);
    
    // Add comment to first move
    game->MoveToPly(1);
    game->SetMoveComment("The most popular opening move.");
    
    // Add another comment
    game->MoveForward();
    game->SetMoveComment("Black mirrors White's central pawn move.");
}

void create_game_with_nags(Game* game, int index) {
    create_simple_game(game, index);
    
    // Add NAG annotations
    game->MoveToPly(1);
    game->AddNag(NAG_GoodMove); // !
    
    game->MoveForward();
    game->AddNag(NAG_PoorMove); // ?
}

void generate_fixture(Game* game, const Options& opts, int index) {
    // Create the game based on type
    switch (opts.type) {
        case SIMPLE:
            create_simple_game(game, index);
            break;
        case VARIATIONS:
            create_game_with_variations(game, index);
            break;
        case COMMENTS:
            create_game_with_comments(game, index);
            break;
        case NAGS:
            create_game_with_nags(game, index);
            break;
        case SPECIAL:
            // TODO: Add edge cases
            create_simple_game(game, index);
            break;
        case FORMATS:
            // TODO: Generate with different formats
            create_simple_game(game, index);
            break;
    }
    
    // Generate PGN output
    TextBuffer tb;
    tb.SetWrapColumn(80);
    game->WriteToPGN(&tb);
    std::string pgn_output = tb.GetBuffer();
    
    // Write fixture to JSON
    char filename[256];
    const char* type_names[] = {"simple", "variations", "comments", "nags", "special", "formats"};
    snprintf(filename, sizeof(filename), "%s/%s_game_%03d.json", 
             opts.output_dir.c_str(), type_names[opts.type], index);
    
    JsonWriter writer(filename);
    writer.WriteFixture(game, pgn_output);
    
    std::cout << "Generated: " << filename << std::endl;
}

int main(int argc, char* argv[]) {
    Options opts;
    opts.type = SIMPLE;
    opts.count = 10;
    opts.output_dir = ".";
    
    // Parse command line options
    static struct option long_options[] = {
        {"type",    required_argument, 0, 't'},
        {"count",   required_argument, 0, 'c'},
        {"output",  required_argument, 0, 'o'},
        {"help",    no_argument,       0, 'h'},
        {0, 0, 0, 0}
    };
    
    int opt;
    int option_index = 0;
    while ((opt = getopt_long(argc, argv, "t:c:o:h", long_options, &option_index)) != -1) {
        switch (opt) {
            case 't':
                opts.type = parse_type(optarg);
                break;
            case 'c':
                opts.count = atoi(optarg);
                break;
            case 'o':
                opts.output_dir = optarg;
                break;
            case 'h':
                print_usage(argv[0]);
                return 0;
            default:
                print_usage(argv[0]);
                return 1;
        }
    }
    
    // Generate fixtures
    Game game;
    game.Init();
    
    for (int i = 1; i <= opts.count; i++) {
        generate_fixture(&game, opts, i);
    }
    
    std::cout << "\nGenerated " << opts.count << " fixtures successfully!" << std::endl;
    
    return 0;
}
