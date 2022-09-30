export type RawContract = Buffer;
export type ArrayifiedContract = string[];
export type StringifiedContract = string;

export interface Variable {
  type: string;
  location: undefined | string;
  name: undefined | string;
}

export interface Line {
  number: number;
  index: number;
  content: string;
  isFirstLine: boolean;
  returns: boolean;
  emits: boolean;
}

export interface Event {
  name: string;
  line: Line;
}

export interface Block {
  type: string; // for-loop | while-loop | unchecked | assembly
  lines: Line[];
}

export type Body = Line[];
export type Declaration = Line[];

export interface Function {
  name: string;
  signature: string;
  selector: string;
  id: number;
  params: Variable[];
  modifiers: string[];
  returns: Variable[];
  body: Body;
  declaration: Declaration;
  firstLine: Line;
  lastLines: Line[];
  events: Event[];
  blocks: Block[];
}
