#ifndef JPSH_EVAL_UTILS_H
#define JPSH_EVAL_UTILS_H

// Removes the element of argv at index idx. Also updates argm and argc.
void rm_element (char **argv, char **argm, int idx, int *argc);

// Adds an element to argv at idx, updating argc and argm.
void add_element (char **argv, char **argm, char *na, char *nm, int idx,
                int *argc);

// An implementation of strchr which will ignore any characters of s
// where the character of m at the same index is any value other than
// '\0'. (that is, it's masked).
char *masked_strchr(const char *s, const char *m, char c);

// The same as above, but for trimming s into ns, and will also trim m
// into nm.
void masked_trim_str(const char *s, const char *m, char **ns, char **nm);
void spl_cmd (const char *s, const char *m, char ***argv, char ***argm,
                int *argc);
/**
 * NOTE:
 *  - WILL thrash arguments
 *
 * Removes the pointed-to character from the string to
 * which it belongs.
 */
void arm_char (char *line, size_t len);

// Prints msg, with every masked character with inverted colors. Allows
// one to pick out bugs in masking.
void print_msg (char *msg, char *mask, int nl);

#endif // JPSH_EVAL_UTILS_H