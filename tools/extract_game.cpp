// Simple tool to extract raw game data from SCID database
// This will dump the binary format so we can test the Rust decoder

#include "common.h"
#include "index.h"
#include "namebase.h"
#include "gfile.h"
#include "game.h"
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char** argv) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s <database> <game_number>\n", argv[0]);
        fprintf(stderr, "Example: %s bases/matein1 1\n", argv[0]);
        return 1;
    }
    
    const char* dbname = argv[1];
    int gameNum = atoi(argv[2]);
    
    if (gameNum < 1) {
        fprintf(stderr, "Error: game number must be >= 1\n");
        return 1;
    }
    
    // Open the index file
    Index idx;
    errorT err = idx.Open(dbname, FMODE_ReadOnly);
    if (err != OK) {
        fprintf(stderr, "Error opening database: %s\n", dbname);
        return 1;
    }
    
    if (gameNum > idx.GetNumGames()) {
        fprintf(stderr, "Error: game %d doesn't exist (db has %u games)\n", 
                gameNum, idx.GetNumGames());
        idx.Close();
        return 1;
    }
    
    // Open game file
    GFile gfile;
    err = gfile.Open(dbname, FMODE_ReadOnly);
    if (err != OK) {
        fprintf(stderr, "Error opening game file\n");
        idx.Close();
        return 1;
    }
    
    // Get game info from index
    IndexEntry* ie = idx.FetchEntry(gameNum - 1);  // 0-indexed
    if (!ie) {
        fprintf(stderr, "Error: couldn't fetch index entry\n");
        gfile.Close();
        idx.Close();
        return 1;
    }
    
    // Read the game data
    ByteBuffer bbuf;
    bbuf.Empty();
    
    err = gfile.ReadGame(&bbuf, ie->GetOffset(), ie->GetLength());
    if (err != OK) {
        fprintf(stderr, "Error reading game data\n");
        gfile.Close();
        idx.Close();
        return 1;
    }
    
    // Output binary data to stdout
    fprintf(stderr, "Game %d: offset=%u length=%u\n", 
            gameNum, ie->GetOffset(), ie->GetLength());
    
    // Write binary data to file
    char outfile[256];
    snprintf(outfile, sizeof(outfile), "game_%d.bin", gameNum);
    FILE* f = fopen(outfile, "wb");
    if (f) {
        fwrite(bbuf.Data(), 1, ie->GetLength(), f);
        fclose(f);
        fprintf(stderr, "Wrote binary game data to: %s\n", outfile);
    }
    
    // Also decode and show as PGN for comparison
    Game game;
    bbuf.Reset();
    err = game.Decode(&bbuf, GAME_DECODE_ALL);
    if (err != OK) {
        fprintf(stderr, "Error decoding game\n");
    } else {
        fprintf(stderr, "\nPGN output:\n");
        fprintf(stderr, "----------------------------------------\n");
        
        game.WritePGN(stdout, &idx, false);
        fprintf(stderr, "\n----------------------------------------\n");
    }
    
    gfile.Close();
    idx.Close();
    
    return 0;
}
