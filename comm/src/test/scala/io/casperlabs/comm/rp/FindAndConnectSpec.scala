package io.casperlabs.comm.rp

import Connect._
import Connections._
import io.casperlabs.comm._
import CommError._
import protocol.routing._
import io.casperlabs.p2p.EffectsTestInstances._
import io.casperlabs.metrics.Metrics

import scala.concurrent.duration._
import org.scalatest._
import org.scalatest.enablers.Containing
import cats.data._
import cats.implicits._
import io.casperlabs.catscontrib._
import Catscontrib._
import cats.Id
import ski._
import io.casperlabs.shared._
import io.casperlabs.comm.transport._
import io.casperlabs.comm.rp.ProtocolHelper._
import io.casperlabs.catscontrib.effect.implicits._

class FindAndConnectSpec extends FunSpec with Matchers with BeforeAndAfterEach with AppendedClues {

  import ScalaTestCats._

  type Effect[A] = EitherT[Id, CommError, A]

  val src: PeerNode              = peer("src")
  val deftimeout: FiniteDuration = FiniteDuration(1, MILLISECONDS)
  implicit val log               = new Log.NOPLog[Id]
  implicit val time              = new LogicalTime[Effect]
  implicit val metric            = new Metrics.MetricsNOP[Id]
  implicit val nodeDiscovery     = new NodeDiscoveryStub[Effect]()
  implicit val rpConf            = conf(defaultTimeout = deftimeout)

  var willConnectSuccessfully       = List.empty[PeerNode]
  var connectCalled: List[PeerNode] = List.empty[PeerNode]

  override def beforeEach(): Unit = {
    willConnectSuccessfully = List.empty[PeerNode]
    nodeDiscovery.nodes = List(peer("A"), peer("B"), peer("C"))
    connectCalled = List.empty[PeerNode]
  }

  val connect: (PeerNode, FiniteDuration) => Effect[Unit] = (peer, to) => {
    connectCalled = connectCalled :+ peer
    if (willConnectSuccessfully.contains(peer)) ().pure[Effect]
    else EitherT[Id, CommError, Unit](Left(timeout))
  }

  describe("Node when called to find and connect") {
    describe("and there are no connections yet") {
      it("should ask NodeDiscovery for the list of peers and try to connect to it") {
        // given
        implicit val connections = mkConnections()
        // when
        Connect.findAndConnect[Effect](connect)
        // then
        connectCalled.size shouldBe (3)
        connectCalled should contain(peer("A"))
        connectCalled should contain(peer("B"))
        connectCalled should contain(peer("C"))
      }

      it("should report peers it connected to successfully") {
        // given
        implicit val connections = mkConnections()
        willConnectSuccessfully = List(peer("A"), peer("C"))
        // when
        val result = Connect.findAndConnect[Effect](connect).value.right.get
        // then
        result.size shouldBe (2)
        result should contain(peer("A"))
        result should not contain (peer("B"))
        result should contain(peer("C"))
      }
    }

    describe("and there already are some connections") {
      it(
        "should ask NodeDiscovery for the list of peers and try to the one he is not connected yet"
      ) {
        // given
        implicit val connections = mkConnections(peer("B"))
        // when
        Connect.findAndConnect[Effect](connect)
        // then
        connectCalled.size shouldBe (2)
        connectCalled should contain(peer("A"))
        connectCalled should contain(peer("C"))
      }

      it("should report peers it connected to successfully") {
        // given
        implicit val connections = mkConnections(peer("B"))
        willConnectSuccessfully = List(peer("A"))
        // when
        val result = Connect.findAndConnect[Effect](connect).value.right.get
        // then
        result.size shouldBe (1)
        result should contain(peer("A"))
        result should not contain (peer("B"))
        result should not contain (peer("C"))
      }

    }

  }

  private def peer(name: String): PeerNode =
    PeerNode(NodeIdentifier(name.getBytes), Endpoint("host", 80, 80))

  private def mkConnections(peers: PeerNode*): ConnectionsCell[Id] =
    Cell.id[Connections](peers.reverse.foldLeft(Connections.empty) {
      case (acc, el) => acc.addConn[Id](el)
    })

  private def conf(
      maxNumOfConnections: Int = 5,
      numOfConnectionsPinged: Int = 5,
      defaultTimeout: FiniteDuration
  ): RPConfAsk[Id] =
    new ConstApplicativeAsk(
      RPConf(
        clearConnections = ClearConnetionsConf(maxNumOfConnections, numOfConnectionsPinged),
        defaultTimeout = defaultTimeout,
        local = peer("src"),
        bootstrap = None
      )
    )

  implicit def eiterTrpConfAsk: RPConfAsk[Effect] =
    new EitherTApplicativeAsk[Id, RPConf, CommError]

}
