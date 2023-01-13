CXX = g++
CXXFLAGS = -Wall -Wextra -pedantic-errors -g -std=c++20
CPPFLAGS =
LDLIBS =
SRCS = language_translator.cpp 
OBJS = $(SRCS:.cpp=.o)
MAIN = language_translator.o

.PHONY: all clean run

all: $(MAIN) 
	echo "Build completed"
$(MAIN): $(OBJS)
	$(CXX) $(CXXFLAGS) $(CPPFLAGS) $(SRCS) -o $(MAIN) 
.cpp.o:
	$(CXX) $(CXXFLAGS) $(CPPFLAGS) -c $< -o $(OBJS)
clean:
	$(RM) *.o *~ $(MAIN)
run: $(MAIN)
	./$(MAIN)
