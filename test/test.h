#define ASSERT(expect, expr) assert(expect, expr, #expr)

int assert(int expect, int actual, char *expr)
{
	if (expect == actual) {
		printf("%s => %d\n", expr, actual);
	} else {
		printf("%s => %d expected, but got %d\n", expr, expect, actual);
		exit(1);
	}
}
