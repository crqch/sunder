defmodule SunderWeb.ApiMacros do
  @moduledoc false

  defmacro authed_operation(action, spec \\ []) do
    responses = Keyword.get(spec, :responses, [])

    unauthorized_response = {
      :%{},
      [],
      [
        type: :object,
        properties: {:%{}, [], [message: {:%{}, [], [type: :string]}]}
      ]
    }

    unauthorized_ast = {unauthorized_response, [description: "Missing/invalid cookie"]}

    new_responses = Keyword.put(responses, :unauthorized, unauthorized_ast)

    security_ast = [{:%{}, [], [{"auth", []}]}]

    merged_spec =
      spec
      |> Keyword.put(:responses, new_responses)
      |> Keyword.put(:security, security_ast)

    quote do
      Oaskit.Controller.operation(
        unquote(action),
        unquote(merged_spec)
      )
    end
  end
end
