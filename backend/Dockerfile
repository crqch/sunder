# Build Stage
FROM hexpm/elixir:1.15.7-erlang-26.1.2-debian-bullseye-20230612-slim AS build

# install build dependencies
RUN apt-get update -y && apt-get install -y build-essential git \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# prepare build dir
WORKDIR /app

# install hex + rebar
RUN mix local.hex --force && \
    mix local.rebar --force

# set build ENV
ENV MIX_ENV=prod

# install mix dependencies
COPY mix.exs mix.lock ./
COPY config config
RUN mix deps.get --only prod
RUN mix deps.compile

# compile assets (skip if API only, but Phoenix 1.5 might have some)
# COPY priv priv
# RUN mix phx.digest

# compile and build release
COPY lib lib
COPY priv priv
RUN mix compile
RUN mix release

# Runtime Stage
FROM debian:bullseye-20230612-slim AS app

RUN apt-get update -y && apt-get install -y libstdc++6 openssl libncurses5 locales \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

# Set the locale
RUN sed -i '/en_US.UTF-8/s/^# //' /etc/locale.gen && locale-gen
ENV LANG=en_US.UTF-8
ENV LANGUAGE=en_US:en
ENV LC_ALL=en_US.UTF-8

WORKDIR /app

RUN chown -R nobody:nogroup /app

USER nobody:nogroup

COPY --from=build --chown=nobody:nogroup /app/_build/prod/rel/sunder ./

ENV HOME=/app

CMD ["bin/sunder", "start"]
