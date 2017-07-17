#ifndef _LIB_OPUS_H_
#define _LIB_OPUS_H_

struct opus_config {

};
typedef struct opus_config opus_config;

void config();
int process(char* record, int len);
int process_from_stream(int fd);

#endif
