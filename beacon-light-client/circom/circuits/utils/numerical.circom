pragma circom 2.1.5;

template DivisionVerification() {
  signal input dividend;
  signal input divisor;
  signal input quotient;
  signal input remainder;

  dividend === divisor * quotient + remainder;
}
