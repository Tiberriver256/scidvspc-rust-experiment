#!/usr/bin/env tclsh

if {$argc < 1} {
    puts stderr "Usage: tclsh get_db_info.tcl <database>"
    exit 1
}

set dbname [lindex $argv 0]

# Load SCID
set scid_dir [file dirname [info script]]
cd $scid_dir

source tcl/start.tcl

if {[catch {sc_base open $dbname} err]} {
    puts stderr "Error opening $dbname: $err"
    exit 1
}

puts "[sc_base numGames]"
sc_base close
