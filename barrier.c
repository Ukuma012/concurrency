#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>

pthread_mutex_t barrier_mut = PTHREAD_MUTEX_INITIALIZER;
pthread_mutex_t barrier_cond = PTHREAD_COND_INITIALIZER;

void barrier(volatile int *cnt, int max)
{
    if (pthread_mutex_lock(&barrier_mut) != 0)
    {
        perror("pthread_mutex_lock");
        exit(-1);
    }

    (*cnt)++;

    if (*cnt == max)
    {
        if (pthread_cond_broadcast(&barrier_cond) != 0)
        {
            perror("pthread_cond_broadcast");
            exit(-1);
        }
    }
    else
    {
        do
        {
            if (pthread_cond_wait(&barrier_cond, &barrier_mut) != 0)
            {
                perror("pthread_cond_wait");
                exit(-1);
            }
        } while (*cnt < max);
    }

    if (pthread_mutex_unlock(&barrier_mut) != 0)
    {
        perror("pthread_mutex_unlock");
        exit(-1);
    }
}
