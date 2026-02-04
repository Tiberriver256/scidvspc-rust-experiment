#!/usr/bin/env wish

# Simple extraction script
if {$argc < 2} {
    puts stderr "Usage: ./tkscid extract.tcl <database> <game_numbers...>"
    exit 1
}

set dbname [lindex $argv 0]
set game_numbers [lrange $argv 1 end]

# Open database
sc_base open $dbname

puts stderr "Database: $dbname"
puts stderr "Total games: [sc_base numGames]"

# Extract games
foreach gnum $game_numbers {
    sc_game load $gnum
    puts "### GAME $gnum ###"
    puts [sc_game pgn -tags 1 -comments 1 -variations 1]
    puts "### END GAME $gnum ###"
    puts ""
}

sc_base close
exit
