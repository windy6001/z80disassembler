//   test program
// name is makecb.c
// by windy

#include <stdio.h>
#include <stdlib.h>

void make_cb(void)
{
	FILE *fp=fopen("cb.bin","w"); if(fp==NULL){printf("file open error\n");exit(0);}
	for(int i=0;i<256;i++) {
		fprintf(fp,"%c%c",0xcb,i);
	}
	fclose(fp);
}

#define m0(op,start,end) {for(int i=start; i<=end;i++){fprintf(fp,"%c%c%c%c",op,i,0x80,0x79);}}
#define m1(op,p0,p1)     {fprintf(fp,"%c%c%c",op,p0,p1);}
#define m2(op,p0,p1,p2)  {fprintf(fp,"%c%c%c%c",op,p0,p1,p2);}


void make_ddfd(void) 
{
	FILE *fp=fopen("ddfd.bin","w"); if(fp==NULL){printf("file open error\n");exit(0);}
	int opcode = 0xdd;
	m0(opcode, 0,0xff);

	opcode = 0xfd;
	m0(opcode, 0,0xff);
	fclose(fp);
}

int main(void){
	make_cb();
	make_ddfd();
}
