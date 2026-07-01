module p #(parameter int WIDTH = 8) (input logic [WIDTH-1:0] d, output logic [WIDTH-1:0] q);
  always_comb q = d;
endmodule
