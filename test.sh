#!/bin/bash

TEST_FN_FILE="tmp_fn.o"

cat <<EOF | cc -xc -c -o "$TEST_FN_FILE" -
int ret3(){return 3;}
int ret5(){return 5;}
int power(int x){return x*x;}
int modulo(int x, int n){return x%n;}
int add6_weight(int x1, int x2, int x3, int x4, int x5, int x6){return x1*1+x2*2+x3*3+x4*4+x5*5+x6*6;}
EOF

error_exit() {
	msg="FAILED"
	printf '\033[31m%s\033[m\n' "$msg"
	exit $1
}

run_test() {
	src="$1"
	echo [`basename "$src"`]

	gcc -xc "$src" -E -P -C | target/debug/rcc - >tmp.s
	[ $? -ne 0 ] && error_exit 1

	gcc -no-pie -o tmp tmp.s "$TEST_FN_FILE"
	[ $? -ne 0 ] && error_exit 1

	./tmp
	[ $? -ne 0 ] && error_exit 1

	echo OK
	echo
}

cargo build || error_exit 1


# 引数がなければtestにある全てのCソースをテストする。
# 引数があれば指定されたソースだけテストする。
if [ $# -eq 0 ]
then
	srcs=`find test -type f -name '*.c' | sort`
else
	srcs="$*"
fi

for src in $srcs
do
	run_test "$src"
done
