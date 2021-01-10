#!/bin/bash

assert() {
	expected="$1"
	input="$2"

	target/debug/rcc "$input" >tmp.s
	cc -o tmp tmp.s
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}

cargo build

assert 0 "0;"
assert 42 "42;"
assert 21 "5+20-4;"
assert 41 " 12 +  34 - 5 ;"
assert 0 "10-  5 -5;"

assert 63 "9 * 7;"
assert 9 "4+3*2-1;"
assert 2 "20/10;"
assert 4 "4+3/2-1;"
assert 5 " 1 + 12 * 4  / 8 - 2 ;"

assert 7 "(4+3)*(2-1);"
assert 8 " ( 1 + 12) * 4  / (8 - 2) ;"
assert 5 " 1 + (((12 * 4)  / 8) - 2) ;"

assert 10 "-10+20;"
assert 60 "+ 15*4;"
assert 1 "- (3 * 2) + 7;"
assert 1 "+(4 / 2) - 1;"

assert 1 "0 == 0;"
assert 0 "0 == 1;"
assert 1 "5+3 == 2*4;"
assert 1 "5+3 == 2*4 == 8-7;"
assert 1 "2 == (4/2+3 == 5) + 1;"
assert 1 "13 != 17;"
assert 0 "19 != 19;"
assert 1 "5*2+3 != 7+5*2 == 1;"

assert 1 "0 < 1;"
assert 0 "1 < 1;"
assert 0 "2 < 1;"
assert 0 "0 > 1;"
assert 0 "1 > 1;"
assert 1 "2 > 1;"
assert 1 "0 <= 1;"
assert 1 "1 <= 1;"
assert 0 "2 <= 1;"
assert 0 "0 >= 1;"
assert 1 "1 >= 1;"
assert 1 "2 >= 1;"
assert 0 "(1+2*3 < 6-5/4) <= 1 != (10 >= 3) + 10 > 9;"

assert 1 "a=1; a;"
assert 2 "z=2; z;"
assert 30 "a=1; b=2; c=3; a=b=c; a*10;"
assert 123 "x = 2 >= 0; y = (x * x) + (x / x); z = -x * (4 - 5) + y; x*100 + y*10 + z;"

assert 72 "val001_num=3*4*6; val002_div=val001_num/6; val003_power=val002_div*val002_div; val003_power/2;"

echo OK
