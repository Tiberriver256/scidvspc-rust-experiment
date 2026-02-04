#!/usr/bin/env tclsh

# Extract games from SCID database to PGN

if {$argc < 2} {
    puts stderr "Usage: tclsh extract_games.tcl <database> <game_numbers...>"
    puts stderr "Example: tclsh extract_games.tcl bases/matein1 1 100 500"
    exit 1
}

# Load SCID Tcl library
set scid_dir [file dirname [info script]]
cd $scid_dir/..

# Source SCID functions
source tcl/start.tcl

set dbname [lindex $argv 0]
set game_numbers [lrange $argv 1 end]

# Open database
if {[catch {sc_base open $dbname} err]} {
    puts stderr "Error opening database '$dbname': $err"
    exit 1
}

puts stderr "Database: $dbname"
puts stderr "Total games: [sc_base numGames]"
puts stderr ""

# Extract each game
foreach gnum $game_numbers {
    puts stderr "Extracting game $gnum..."
    
    if {[catch {sc_game load $gnum} err]} {
        puts stderr "Error loading game $gnum: $err"
        continue
    }
    
    puts "### GAME $gnum ###"
    puts [sc_game pgn -tags 1 -comments 1 -variations 1]
    puts "### END GAME $gnum ###"
    puts ""
}

sc_base close
puts stderr "Done."
