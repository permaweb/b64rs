-module(b64rs).

-export([encode/1, decode/1]).

-on_load(init/0).

-define(APPNAME, ?MODULE).
-define(LIBNAME, ?MODULE).

encode(Bin) when is_binary(Bin) ->
    nif_encode(Bin);
encode(L) when is_list(L) ->
    nif_encode(iolist_to_binary(L));
encode(_) ->
    error(badarg).

decode(Bin) when is_binary(Bin) ->
    nif_decode(Bin);
decode(L) when is_list(L) ->
    nif_decode(iolist_to_binary(L));
decode(_) ->
    error(badarg).

nif_encode(_Bin) ->
    not_loaded(?LINE).

nif_decode(_Bin) ->
    not_loaded(?LINE).

init() ->
    SoName = case code:priv_dir(?APPNAME) of
        {error, bad_name} ->
            case filelib:is_dir(filename:join(["..", priv])) of
                true ->
                    filename:join(["..", priv, ?LIBNAME]);
                _ ->
                    filename:join([priv, ?LIBNAME])
            end;
        Dir ->
            filename:join(Dir, ?LIBNAME)
    end,
    erlang:load_nif(SoName, 0).

not_loaded(Line) ->
    erlang:nif_error({not_loaded, [{module, ?MODULE}, {line, Line}]}).
