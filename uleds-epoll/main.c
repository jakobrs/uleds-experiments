#include <stdio.h>
#include <sys/epoll.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <linux/uleds.h>
#include <stdlib.h>

int main() {
  int epollfd = epoll_create1(0);

  struct uleds_user_dev uled_decl;
  uled_decl.max_brightness = 1000;

  int ledfds[1000];
  for (int n = 0; n < 1000; n++) {
    int fd = open("/dev/uleds", O_RDWR);
    if (fd == -1) {
      perror("Error opening /dev/uleds");
      exit(-1);
    }

    snprintf(uled_decl.name, LED_MAX_NAME_SIZE, "neopixel::led%d", n);
    int count = write(fd, &uled_decl, sizeof(struct uleds_user_dev));
    if (count == -1) {
      perror("Error writing struct to /dev/uleds");
      exit(-1);
    }

    struct epoll_event events = {
      .events = EPOLLIN,
      .data = { .u32 = n },
    };
    int err = epoll_ctl(epollfd, EPOLL_CTL_ADD, fd, &events);
    if (err == -1) {
      perror("Error calling epoll_ctl");
      exit(-1);
    }

    ledfds[n] = fd;
  }

  while (1) {
    struct epoll_event events;
    int event_count = epoll_wait(epollfd, &events, 1, -1);

    if (event_count == -1) {
      perror("Error while calling epoll_wait");
      exit(-1);
    } else if (event_count < 1) {
      printf("Didn't receive event\n");
      exit(-1);
    }

    int n = events.data.u32;
    printf("Received events %d on fd %d\n", events.events, n);

    uint32_t brightness;
    int count = read(ledfds[n], &brightness, 4);

    if (count == -1) {
      perror("Error while reading from fd");
      exit(-1);
    } else if (count < 4) {
      printf("Didn't receive entire number\n");
      exit(-1);
    }
    printf("Brightness set to %d\n", brightness);

    // Here you would call a function to actually set the brightness
  }

  sleep(1);
}
