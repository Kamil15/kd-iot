FROM mcr.microsoft.com/dotnet/sdk:8.0 AS build-env
WORKDIR /App

# Copy everything
COPY . ./
# Restore as distinct layers
RUN dotnet restore
# Build and publish a release
RUN dotnet publish -c Release -o out

# Build runtime image
FROM mcr.microsoft.com/dotnet/aspnet:8.0-alpine
RUN apk --no-cache add ca-certificates


WORKDIR /App
COPY --from=build-env /App/out .

# COPY rabbitmq-etc/ca_certificate.pem /usr/local/share/ca-certificates/ca_certificate.pem
# RUN update-ca-certificates
ENTRYPOINT ["dotnet", "KdIoT.Server.dll"]