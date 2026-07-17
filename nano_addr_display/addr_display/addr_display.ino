#include <TM1637Display.h>

// Module connection pins (Digital Pins)
#define CLK 3
#define DIO 2

TM1637Display display(CLK, DIO);

void displayNum(uint32_t num){
  bool colon = (num >> 17) & 1;
  display.showNumberHexEx(num, colon ? 0b11100000 : 0, true, 4, 0);
}

// one  ore
uint8_t addressMap[16] = { 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, A1, A2, A3, A4, A5, A6};

// have just enough pins left for 17 address (17th bit being displayed as the colon)
// uint8_t addressMap[17] = { 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, A1, A2, A3, A4, A5, A6, A7};

uint32_t getAddress(){
  uint32_t result;
  for(int i = 0;i < sizeof(addressMap) / sizeof(addressMap[0]);i++){
    if(digitalRead(addressMap[i])) result |= 1 << i;
  } 
  return result;
}

void setup() {
  for(int i = 0;i < sizeof(addressMap) / sizeof(addressMap[0]);i++){
    pinMode(addressMap[i], INPUT);
  }
}

void loop() {
  // put your setup code here, to run once:
  display.setBrightness(1);
  displayNum(0x5a5a);
  //displayNum(getAddress());

  delay(10);
}
