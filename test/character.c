#include "test.h"

int main()
{
	ASSERT(48, '0');
	ASSERT(64, '@');
	ASSERT(65, 'A');
	ASSERT(122, 'z');
	ASSERT(126, '~');

	ASSERT(7, '\a');
	ASSERT(8, '\b');
	ASSERT(12, '\f');
	ASSERT(10, '\n');
	ASSERT(13, '\r');
	ASSERT(9, '\t');
	ASSERT(8, '\v');
	ASSERT(27, '\e');
	ASSERT(34, '\"');
	ASSERT(34, '"');
	ASSERT(63, '\?');
	ASSERT(92, '\\');
	ASSERT(39, '\'');

	ASSERT(10, '\xa');
	ASSERT(10, '\xA');
	ASSERT(-85, '\x0aB');
	ASSERT(-85, '\x0Ab');
	ASSERT(-1, '\x00ff');

	ASSERT(0, '\0');
	ASSERT(7, '\007');
	ASSERT(87, '\127');

	return 0;
}
