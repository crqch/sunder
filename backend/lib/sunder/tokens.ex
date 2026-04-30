defmodule Sunder.Tokens do
  def generate_token(byte_size \\ 32) do
    byte_size
    |> :crypto.strong_rand_bytes()
    |> Base.url_encode64(padding: false)
  end
end
