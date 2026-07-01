interface bus;
  logic req;
  logic gnt;
  modport master (output req, input gnt);
endinterface
