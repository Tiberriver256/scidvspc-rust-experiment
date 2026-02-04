/*
 * JSON Writer Implementation
 */

#include "json_writer.h"
#include <iostream>
#include <sstream>

JsonWriter::JsonWriter(const std::string& filename) : indent_level(0) {
    file.open(filename);
    if (!file.is_open()) {
        std::cerr << "Failed to open file: " << filename << std::endl;
        exit(1);
    }
}

JsonWriter::~JsonWriter() {
    if (file.is_open()) {
        file.close();
    }
}

void JsonWriter::write_indent() {
    for (int i = 0; i < indent_level; i++) {
        file << "  ";
    }
}

void JsonWriter::escape_string(const std::string& str, std::string& escaped) {
    escaped.clear();
    for (char c : str) {
        switch (c) {
            case '"':  escaped += "\\\""; break;
            case '\\': escaped += "\\\\"; break;
            case '\n': escaped += "\\n"; break;
            case '\r': escaped += "\\r"; break;
            case '\t': escaped += "\\t"; break;
            default:   escaped += c; break;
        }
    }
}

void JsonWriter::WriteFixture(Game* game, const std::string& expected_pgn) {
    file << "{\n";
    indent_level++;
    
    // Name
    write_indent();
    file << "\"name\": \"test_" << game->GetEventStr() << "\",\n";
    
    // Description
    write_indent();
    file << "\"description\": \"" << game->GetEventStr() << " - "
         << game->GetWhiteStr() << " vs " << game->GetBlackStr() << "\",\n";
    
    // Input
    write_indent();
    file << "\"input\": ";
    WriteGameInput(game);
    file << ",\n";
    
    // Expected PGN
    write_indent();
    file << "\"expected_pgn\": \"";
    std::string escaped;
    escape_string(expected_pgn, escaped);
    file << escaped << "\",\n";
    
    // Options
    write_indent();
    file << "\"options\": ";
    WriteOptions(game);
    file << "\n";
    
    indent_level--;
    file << "}\n";
}

void JsonWriter::WriteGameInput(Game* game) {
    file << "{\n";
    indent_level++;
    
    write_indent();
    file << "\"event\": \"" << game->GetEventStr() << "\",\n";
    
    write_indent();
    file << "\"site\": \"" << game->GetSiteStr() << "\",\n";
    
    write_indent();
    char dateStr[20];
    date_DecodeToString(game->GetDate(), dateStr);
    file << "\"date\": \"" << dateStr << "\",\n";
    
    write_indent();
    file << "\"round\": \"" << game->GetRoundStr() << "\",\n";
    
    write_indent();
    file << "\"white\": \"" << game->GetWhiteStr() << "\",\n";
    
    write_indent();
    file << "\"black\": \"" << game->GetBlackStr() << "\",\n";
    
    write_indent();
    file << "\"result\": \"" << RESULT_LONGSTR[game->GetResult()] << "\",\n";
    
    write_indent();
    if (game->GetWhiteElo() > 0) {
        file << "\"white_elo\": " << game->GetWhiteElo() << ",\n";
        write_indent();
    }
    
    if (game->GetBlackElo() > 0) {
        file << "\"black_elo\": " << game->GetBlackElo() << ",\n";
        write_indent();
    }
    
    if (game->GetEco() != 0) {
        ecoStringT ecoStr;
        eco_ToExtendedString(game->GetEco(), ecoStr);
        file << "\"eco\": \"" << ecoStr << "\",\n";
        write_indent();
    }
    
    // Moves (simplified for now)
    file << "\"moves\": []";
    
    // TODO: Add variations, comments, NAGs
    
    file << "\n";
    indent_level--;
    write_indent();
    file << "}";
}

void JsonWriter::WriteOptions(Game* game) {
    file << "{\n";
    indent_level++;
    
    write_indent();
    const char* format_names[] = {"plain", "html", "latex", "color"};
    file << "\"format\": \"" << format_names[game->GetPgnFormat()] << "\",\n";
    
    write_indent();
    file << "\"include_comments\": " 
         << ((game->GetPgnStyle() & PGN_STYLE_COMMENTS) ? "true" : "false") << ",\n";
    
    write_indent();
    file << "\"include_variations\": " 
         << ((game->GetPgnStyle() & PGN_STYLE_VARS) ? "true" : "false") << ",\n";
    
    write_indent();
    file << "\"include_nags\": true,\n";
    
    write_indent();
    file << "\"short_header\": " 
         << ((game->GetPgnStyle() & PGN_STYLE_SHORT_HEADER) ? "true" : "false") << "\n";
    
    indent_level--;
    write_indent();
    file << "}";
}
