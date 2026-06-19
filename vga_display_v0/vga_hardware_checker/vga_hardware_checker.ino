const int CLK = 2;
const int VSYNC = 3;
const int HSYNC = 4;
const int HRESET = 5;
const int VRESET = 6;

typedef struct {
  uint64_t clock_cycle;
  int x;
  int y;
  int vsync;
  int hsync;
  int vreset;
  int hreset;

} event_t;

event_t last_event, curr_event;

void setup() {
  // put your setup code here, to run once:

  Serial.begin(115200);
  while (!Serial) delay(10);
  pinMode(CLK, OUTPUT);

  pinMode(VSYNC, INPUT);
  pinMode(HSYNC, INPUT);
  pinMode(VRESET, INPUT);
  pinMode(HRESET, INPUT);

  curr_event.clock_cycle = 0;
  curr_event.x = 0;
  curr_event.y = 0;
  curr_event.hreset = 1;
  curr_event.vreset = 1;
  curr_event.hsync = 1;
  curr_event.vsync = 1;

  last_event = curr_event;
}


void print_uint64_t(uint64_t num) {
  char rev[128];
  char *p = rev + 1;

  while (num > 0) {
    *p++ = '0' + (num % 10);
    num /= 10;
  }
  p--;
  /*Print the number which is now in reverse*/
  while (p > rev) {
    Serial.print(*p--);
  }
}


inline bool eventChanged() {
  if (last_event.vsync != curr_event.vsync) return true;
  if (last_event.hsync != curr_event.hsync) return true;
  if (last_event.vreset != curr_event.vreset) return true;
  if (last_event.hreset != curr_event.hreset) return true;
  return false;
}

inline void printCurrent() {
  print_uint64_t(curr_event.clock_cycle);
  Serial.print(",");
  Serial.print(curr_event.x);
  Serial.print(",");
  Serial.print(curr_event.y);
  Serial.print(",");
  Serial.print(curr_event.vsync);
  Serial.print(",");
  Serial.print(curr_event.hsync);
  Serial.print(",");
  Serial.print(curr_event.vreset);
  Serial.print(",");
  Serial.print(curr_event.hreset);
  Serial.println();
}

void loop() {
  digitalWrite(CLK, HIGH);
  digitalWrite(CLK, LOW);

  curr_event.x++;
  curr_event.clock_cycle++;

  curr_event.vsync = digitalRead(VSYNC) == LOW;
  curr_event.hsync = digitalRead(HSYNC) == LOW;
  curr_event.vreset = digitalRead(VRESET) == LOW;
  curr_event.hreset = digitalRead(HRESET) == LOW;

  if (eventChanged()) {
    printCurrent();
    last_event = curr_event;
  }

  if (curr_event.hreset) {
    curr_event.x = 0;
    curr_event.y++;
  }
  if (curr_event.vreset) {
    curr_event.y = 0;
  }
}
