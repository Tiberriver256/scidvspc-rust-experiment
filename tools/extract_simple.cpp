// Minimal C++ extractor - reads SCID database and outputs PGN

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern "C" {
    #include <tcl.h>
}

int main(int argc, char** argv) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s <database> <game_numbers...>\n", argv[0]);
        fprintf(stderr, "Example: %s bases/matein1 1 100 500\n", argv[0]);
        return 1;
    }
    
    // Initialize Tcl interpreter
    Tcl_Interp *interp = Tcl_CreateInterp();
    if (Tcl_Init(interp) != TCL_OK) {
        fprintf(stderr, "Error initializing Tcl: %s\n", Tcl_GetStringResult(interp));
        return 1;
    }
    
    // Load SCID commands (assuming scid library is available)
    // We'll use a simpler approach - just call scid's tcl functions
    
    const char* dbname = argv[1];
    
    // For now, let's just create a TCL script that uses scid
    printf("#!/usr/bin/env tclsh\n\n");
    printf("# Auto-generated extraction script\n");
    printf("# Load scid\n");
    printf("load ./scid.so\n\n");
    
    for (int i = 2; i < argc; i++) {
        printf("# Extract game %s\n", argv[i]);
        printf("sc_base open %s\n", dbname);
        printf("sc_game load %s\n", argv[i]);
        printf("puts \"### GAME %s ###\"\n", argv[i]);
        printf("puts [sc_game pgn -tags 1 -comments 1 -variations 1]\n");
        printf("puts \"### END GAME %s ###\\n\"\n", argv[i]);
        printf("sc_base close\n\n");
    }
    
    Tcl_DeleteInterp(interp);
    return 0;
}
