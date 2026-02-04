# TCL script to extract games from SCID database

proc extract_game {dbname gnum} {
    # Open database
    if {[catch {sc_base open $dbname} err]} {
        puts stderr "Error opening database: $err"
        return ""
    }
    
    # Load game
    if {[catch {sc_game load $gnum} err]} {
        puts stderr "Error loading game $gnum: $err"
        sc_base close
        return ""
    }
    
    # Get PGN
    set pgn [sc_game pgn -tags 1 -comments 1 -variations 1]
    
    # Close database
    sc_base close
    
    return $pgn
}

# Main
if {$argc < 2} {
    puts stderr "Usage: tclsh extract_games.tcl <database> <game_numbers...>"
    exit 1
}

set dbname [lindex $argv 0]
set game_numbers [lrange $argv 1 end]

foreach gnum $game_numbers {
    puts "### GAME $gnum ###"
    puts [extract_game $dbname $gnum]
    puts "### END GAME $gnum ###"
    puts ""
}
