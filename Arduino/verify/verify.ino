#include "import.hh"

bool isTrue;
int LoopCounter = 0;
void setup() {
  Serial.begin(9600);  
  Serial.printf("TESTING");

  pinMode(LED_BUILTIN, OUTPUT);
  digitalWrite(LED_BUILTIN, LOW);
  delay(2000);

  isTrue = makePairsAndVerify(&vkey,
                                 &proof,
                                 &currentHeader,
                                 &newOptimisticHeader,
                                 &newFinalizedHeader,
                                 &newExecutionStateRoot,
                                 &new_slot,
                                 &domain);
  Serial.printf("End of Setup");


}


void loop() {
  Serial.println("Looping");
  LoopCounter++;
  
  Serial.print("LoopCounter value = ");
  Serial.println(LoopCounter);
  Serial.println("Print 1 if function returns ");
  if(isTrue)
  {
    printf("1");
  }

  digitalWrite(LED_BUILTIN, LOW);
  delay(2000);

  digitalWrite(LED_BUILTIN, HIGH);
  delay(2000);

}
