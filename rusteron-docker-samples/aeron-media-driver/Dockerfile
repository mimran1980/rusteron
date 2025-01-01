FROM maven:3.8.6-openjdk-11 as builder

ENV AERON_VERSION=1.46.7
WORKDIR /aeron

RUN mvn org.apache.maven.plugins:maven-dependency-plugin:3.8.1:copy \
    -DrepoUrl=https://repo1.maven.org/maven2/ \
    -Dartifact=io.aeron:aeron-all:${AERON_VERSION} -DoutputDirectory=/aeron && \
    mvn org.apache.maven.plugins:maven-dependency-plugin:3.8.1:copy \
    -DrepoUrl=https://repo1.maven.org/maven2/ \
    -Dartifact=io.aeron:aeron-archive:${AERON_VERSION} -DoutputDirectory=/aeron

FROM eclipse-temurin:21

WORKDIR /aeron
COPY --from=builder /aeron/*.jar /aeron

EXPOSE 8010 8011 8012

ENV AERON_ARCHIVE_DIR="/data/archive"
ENV AERON_EVENT_LOG="all"
ENV AERON_EVENT_ARCHIVE_LOG="all"
ENV AERON_TERM_BUFFER_SPARSE_FILE="false"
ENV AERON_PRE_TOUCH_MAPPED_MEMORY="true"
ENV AERON_THREADING_MODE="DEDICATED"
ENV AERON_SENDER_IDLE_STRATEGY="noop"
ENV AERON_RECEIVER_IDLE_STRATEGY="noop"
ENV AERON_CONDUCTOR_IDLE_STRATEGY="spin"
ENV AERON_DISABLE_BOUNDS_CHECKS="true"
ENV AERON_DIR_DELETE_ON_START="true"
ENV AERON_DIR_DELETE_ON_SHUTDOWN="true"
ENV AERON_PRINT_CONFIGURATION="true"
ENV AERON_ARCHIVE_CONTROL_CHANNEL="aeron:udp?endpoint=localhost:8010"
ENV AERON_ARCHIVE_REPLICATION_CHANNEL="aeron:udp?endpoint=localhost:8011"
ENV AERON_ARCHIVE_CONTROL_RESPONSE_CHANNEL="aeron:udp?endpoint=localhost:8012"

# CMD using environment variables to pass options to the Java application
CMD ["sh", "-c", "java -cp '/aeron/*' \
  -Daeron.archive.dir=${AERON_ARCHIVE_DIR} \
  -Daeron.event.log=${AERON_EVENT_LOG} \
  -Daeron.event.archive.log=${AERON_EVENT_ARCHIVE_LOG} \
  -Daeron.term.buffer.sparse.file=${AERON_TERM_BUFFER_SPARSE_FILE} \
  -Daeron.pre.touch.mapped.memory=${AERON_PRE_TOUCH_MAPPED_MEMORY} \
  -Daeron.threading.mode=${AERON_THREADING_MODE} \
  -Daeron.sender.idle.strategy=${AERON_SENDER_IDLE_STRATEGY} \
  -Daeron.receiver.idle.strategy=${AERON_RECEIVER_IDLE_STRATEGY} \
  -Daeron.conductor.idle.strategy=${AERON_CONDUCTOR_IDLE_STRATEGY} \
  -Dagrona.disable.bounds.checks=${AERON_DISABLE_BOUNDS_CHECKS} \
  -Daeron.dir.delete.on.start=${AERON_DIR_DELETE_ON_START} \
  -Daeron.dir.delete.on.shutdown=${AERON_DIR_DELETE_ON_SHUTDOWN} \
  -Daeron.print.configuration=${AERON_PRINT_CONFIGURATION} \
  -Daeron.archive.control.channel=${AERON_ARCHIVE_CONTROL_CHANNEL} \
  -Daeron.archive.replication.channel=${AERON_ARCHIVE_REPLICATION_CHANNEL} \
  -Daeron.archive.control.response.channel=${AERON_ARCHIVE_CONTROL_RESPONSE_CHANNEL} \
  io.aeron.archive.ArchivingMediaDriver"]