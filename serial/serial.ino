/*
  Input Pull-up Serial + Debounce

  This example demonstrates the use of pinMode(INPUT_PULLUP). It reads a digital
  input on pin 2 and prints the results to the Serial Monitor.

  The circuit:
  - momentary switch attached from pin 2 to ground
  - built-in LED on pin 13

  Unlike pinMode(INPUT), there is no pull-down resistor necessary. An internal
  20K-ohm resistor is pulled to 5V. This configuration causes the input to read
  HIGH when the switch is open, and LOW when it is closed.

  created 14 Mar 2012
  by Scott Fitzgerald
  modified Jan 2021 by Michael Farrell <micolous+git@gmail.com>

  This example code is in the public domain.

  http://www.arduino.cc/en/Tutorial/InputPullupSerial
*/

const int buttonPin = 2;    // the number of the pushbutton pin
#ifdef ARDUINO_AVR_PROMICRO
// "Pro Micro" boards don't have an LED wired to the "usual" pin 13,
// use the RX LED (17) instead.
// https://learn.sparkfun.com/tutorials/pro-micro--fio-v3-hookup-guide/example-1-blinkies
const int ledPin = 17;
#else
const int ledPin = LED_BUILTIN;
#endif

int ledState = LOW;          // the current state of the output pin
int buttonState;             // the current reading from the input pin
int lastButtonState = LOW;   // the previous reading from the input pin

// the following variables are unsigned longs because the time, measured in
// milliseconds, will quickly become a bigger number than can be stored in an int.
unsigned long lastDebounceTime = 0;  // the last time the output pin was toggled
unsigned long debounceDelay = 50;    // the debounce time; increase if the output flickers


void setup() {
  //start serial connection
  Serial.begin(9600);
  //configure pin 2 as an input and enable the internal pull-up resistor
  pinMode(buttonPin, INPUT_PULLUP);
  pinMode(ledPin, OUTPUT);

}

void loop() {
  //read the pushbutton value into a variable
  int sensorVal = digitalRead(2);

  // check to see if you just pressed the button
  // (i.e. the input went from LOW to HIGH), and you've waited long enough
  // since the last press to ignore any noise:

  // If the switch changed, due to noise or pressing:
  if (sensorVal != lastButtonState) {
    // reset the debouncing timer
    lastDebounceTime = millis();
  }

  if ((millis() - lastDebounceTime) > debounceDelay) {
    // whatever the reading is at, it's been there for longer than the debounce
    // delay, so take it as the actual current state:

    // if the button state has changed:
    if (sensorVal != buttonState) {
      buttonState = sensorVal;

      // Keep in mind the pull-up means the pushbutton's logic is inverted. It goes
      // HIGH when it's open, and LOW when it's pressed.
      Serial.print(buttonState == LOW ? "1" : "0");
#ifdef ARDUINO_AVR_PROMICRO
      // Pro Micro has inverted LED state (LOW = on)
      ledState = buttonState == HIGH;
#else
      ledState = buttonState == LOW;
#endif
    }
  }

  // set the LED:
  digitalWrite(ledPin, ledState);

  // save the reading. Next time through the loop, it'll be the lastButtonState:
  lastButtonState = sensorVal;
}
