#include <string>
#ifndef UTILS_H
#define UTILS_H
std::string remove_whitespaces(std::string& s){
	//traversing the string
	for (int i = 0; i < s.length(); i++){
		if (s[i] == ' '){
			s.erase(s.begin() + i);
			i--;
		}
	}
	return s;
}
#endif
