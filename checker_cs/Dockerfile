FROM mcr.microsoft.com/dotnet/sdk:5.0 AS build
WORKDIR /src

# Fetch dependencies
COPY N0t3b00kChecker.sln .
COPY N0t3b00kChecker/N0t3b00kChecker.csproj N0t3b00kChecker/N0t3b00kChecker.csproj
RUN dotnet restore

# Publish
COPY . .
RUN dotnet publish -c Release --self-contained true -o /build

# Copy to runtime container
FROM ghcr.io/enowars/enochecker-dotnet:latest
COPY --from=build /build /N0t3b00kChecker

CMD ["/N0t3b00kChecker/N0t3b00kChecker.dll"]
