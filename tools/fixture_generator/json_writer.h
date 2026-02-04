/*
 * JSON Writer for Test Fixtures
 * 
 * Outputs game data in JSON format for cross-language testing
 */

#ifndef JSON_WRITER_H
#define JSON_WRITER_H

#include <string>
#include <fstream>
#include "game.h"

class JsonWriter {
private:
    std::ofstream file;
    int indent_level;
    
    void write_indent();
    void escape_string(const std::string& str, std::string& escaped);
    
public:
    JsonWriter(const std::string& filename);
    ~JsonWriter();
    
    void WriteFixture(Game* game, const std::string& expected_pgn);
    void WriteGameInput(Game* game);
    void WriteOptions(Game* game);
};

#endif // JSON_WRITER_H
