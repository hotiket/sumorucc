#include "test.h"

char *g0="GVar";
char *g1[2]={"Hello", "world!"};

int main()
{
	ASSERT(99, ({char *p = "A"; 99;}));
	ASSERT(66, ({char b="ABC"[1]; b;}));
	ASSERT(0, ({char null="123"[3]; null;}));
	ASSERT(3, ({char *p="Hello"; char *q="String"; q[0] + q[5] - p[0] - p[4];}));
	ASSERT(13, sizeof "Hello world!");
	ASSERT(185, g0[0] + g0[3] + g0[4]);
	ASSERT(144, g1[0][4] + g1[1][5]);

	ASSERT(7, "\a\b\f\n\r\t\v\e"[0]);
	ASSERT(8, "\a\b\f\n\r\t\v\e"[1]);
	ASSERT(12, "\a\b\f\n\r\t\v\e"[2]);
	ASSERT(10, "\a\b\f\n\r\t\v\e"[3]);
	ASSERT(13, "\a\b\f\n\r\t\v\e"[4]);
	ASSERT(9, "\a\b\f\n\r\t\v\e"[5]);
	ASSERT(8, "\a\b\f\n\r\t\v\e"[6]);
	ASSERT(27, "\a\b\f\n\r\t\v\e"[7]);
	ASSERT(34, "\"\?\\"[0]);
	ASSERT(63, "\"\?\\"[1]);
	ASSERT(92, "\"\?\\"[2]);
	ASSERT(39, "\'"[0]);
	ASSERT(198, ({char *p="\A\B\C"; p[0] + p[1] + p[2] + p[3];}));

	ASSERT(10, "\xa"[0]);
	ASSERT(10, "\xA"[0]);
	ASSERT(-85, "\x0aB"[0]);
	ASSERT(-85, "\x0Abx"[0]);
	ASSERT(120, "\x0Abx"[1]);
	ASSERT(-1, "\x00ff"[0]);

	ASSERT(0, "\0"[0]);
	ASSERT(7, "\007"[0]);
	ASSERT(87, "\127"[0]);
	ASSERT(48, "\1500"[1]);

	return 0;
}
