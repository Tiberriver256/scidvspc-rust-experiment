# Test Fixture Generator for SCID PGN Conversion

SCID_SRC = ../../src
CXX = g++
CXXFLAGS = -I$(SCID_SRC) -std=c++11 -Wall -Wextra
LDFLAGS = 

# Source files from SCID
SCID_OBJS = \
	$(SCID_SRC)/game.o \
	$(SCID_SRC)/position.o \
	$(SCID_SRC)/textbuf.o \
	$(SCID_SRC)/bytebuf.o \
	$(SCID_SRC)/stored.o \
	$(SCID_SRC)/date.o \
	$(SCID_SRC)/namebase.o \
	$(SCID_SRC)/matsig.o

# Our fixture generator
FIXTURE_GEN_OBJS = main.o json_writer.o

TARGET = fixture_generator

.PHONY: all clean

all: $(TARGET)

$(TARGET): $(FIXTURE_GEN_OBJS) $(SCID_OBJS)
	$(CXX) $(CXXFLAGS) -o $@ $^ $(LDFLAGS)

%.o: %.cpp
	$(CXX) $(CXXFLAGS) -c $< -o $@

clean:
	rm -f $(TARGET) $(FIXTURE_GEN_OBJS)

# Note: This Makefile assumes SCID objects are already built
# You may need to adjust paths and dependencies based on your setup
