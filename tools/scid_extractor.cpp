// C++ tool to extract games from SCID database and output PGN
// This serves as the "oracle" for testing the Rust implementation

#include "common.h"
#include "index.h"
#include "namebase.h"
#include "gfile.h"
#include "game.h"
#include "textbuf.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <vector>

void usage(const char* progname) {
    fprintf(stderr, "Usage: %s <database> <game_numbers...>\n", progname);
    fprintf(stderr, "Example: %s bases/matein1 1 100 500\n", progname);
    fprintf(stderr, "\n");
    fprintf(stderr, "Extracts specified games and outputs them as PGN.\n");
    fprintf(stderr, "Use 'random N' to extract N random games.\n");
    exit(1);
}

int main(int argc, char** argv) {
    if (argc < 3) {
        usage(argv[0]);
    }
    
    const char* dbname = argv[1];
    
    // Initialize namebase and index
    NameBase* nb = new NameBase;
    Index idx;
    
    // Read namebase
    if (nb->ReadNameFile(dbname) != OK) {
        fprintf(stderr, "Error: Cannot open namebase '%s'\n", dbname);
        delete nb;
        return 1;
    }
    
    // Set up index with namebase
    idx.SetFileName(dbname);
    idx.SetDescription("Extracted games");
    
    // Open index file
    if (idx.OpenIndexFile(FMODE_ReadOnly) != OK) {
        fprintf(stderr, "Error: Cannot open index file '%s'\n", dbname);
        nb->Clear();
        delete nb;
        return 1;
    }
    
    // Read entire index
    if (idx.ReadEntireFile() != OK) {
        fprintf(stderr, "Error: Cannot read index file '%s'\n", dbname);
        idx.CloseIndexFile();
        nb->Clear();
        delete nb;
        return 1;
    }
    
    // Open game file
    GFile gfile;
    if (gfile.Open(dbname, FMODE_ReadOnly) != OK) {
        fprintf(stderr, "Error: Cannot open game file '%s'\n", dbname);
        idx.CloseIndexFile();
        nb->Clear();
        delete nb;
        return 1;
    }
    
    uint numGames = idx.GetNumGames();
    fprintf(stderr, "Database: %s\n", dbname);
    fprintf(stderr, "Total games: %u\n\n", numGames);
    
    // Parse game numbers
    std::vector<uint> gameNumbers;
    
    if (strcmp(argv[2], "random") == 0) {
        // Random games
        if (argc < 4) {
            fprintf(stderr, "Error: 'random' requires number of games\n");
            usage(argv[0]);
        }
        
        int count = atoi(argv[3]);
        if (count <= 0 || count > (int)numGames) {
            fprintf(stderr, "Error: Invalid count %d (must be 1-%u)\n", count, numGames);
            gfile.Close();
            idx.CloseIndexFile();
            nb->Clear();
            delete nb;
            return 1;
        }
        
        // Seed random number generator
        srand(time(NULL));
        
        // Generate random game numbers
        for (int i = 0; i < count; i++) {
            uint gnum = (rand() % numGames) + 1;
            gameNumbers.push_back(gnum);
        }
    } else {
        // Specific game numbers
        for (int i = 2; i < argc; i++) {
            int gnum = atoi(argv[i]);
            if (gnum < 1 || gnum > (int)numGames) {
                fprintf(stderr, "Warning: Game %d out of range (1-%u), skipping\n", 
                        gnum, numGames);
                continue;
            }
            gameNumbers.push_back(gnum);
        }
    }
    
    if (gameNumbers.empty()) {
        fprintf(stderr, "Error: No valid game numbers specified\n");
        gfile.Close();
        idx.CloseIndexFile();
        nb->Clear();
        delete nb;
        return 1;
    }
    
    // Extract games
    Game* game = new Game;
    TextBuffer tb;
    
    for (size_t i = 0; i < gameNumbers.size(); i++) {
        uint gnum = gameNumbers[i];
        
        fprintf(stderr, "Extracting game %u...\n", gnum);
        
        // Get index entry (0-based)
        IndexEntry* ie = idx.FetchEntry(gnum - 1);
        if (!ie) {
            fprintf(stderr, "Error: Cannot fetch index entry for game %u\n", gnum);
            continue;
        }
        
        // Read game data
        ByteBuffer bbuf;
        bbuf.Empty();
        
        if (gfile.ReadGame(&bbuf, ie->GetOffset(), ie->GetLength()) != OK) {
            fprintf(stderr, "Error: Cannot read game %u data\n", gnum);
            continue;
        }
        
        // Decode game
        bbuf.BackToStart();
        if (game->Decode(&bbuf, GAME_DECODE_ALL) != OK) {
            fprintf(stderr, "Error: Cannot decode game %u\n", gnum);
            continue;
        }
        
        // Load standard tags from index
        game->LoadStandardTags(ie, nb);
        
        // Output separator
        printf("### GAME %u ###\n", gnum);
        
        // Write PGN to text buffer
        tb.Empty();
        game->WriteToPGN(&tb);
        
        // Output the PGN
        printf("%s\n", tb.GetBuffer());
        
        // Output end marker
        printf("### END GAME %u ###\n\n", gnum);
    }
    
    // Cleanup
    delete game;
    gfile.Close();
    idx.CloseIndexFile();
    nb->Clear();
    delete nb;
    
    fprintf(stderr, "\nExtracted %zu games successfully.\n", gameNumbers.size());
    
    return 0;
}
