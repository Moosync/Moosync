package app.moosync.android

import android.content.Intent
import android.util.Log
import org.junit.AfterClass
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.BeforeClass
import org.junit.Test
import org.mockito.MockedStatic
import org.mockito.Mockito

class HandlerUnitTests {

    companion object {
        private var mockedLog: MockedStatic<Log>? = null

        @JvmStatic
        @BeforeClass
        fun setUp() {
            mockedLog = Mockito.mockStatic(Log::class.java)
        }

        @JvmStatic
        @AfterClass
        fun tearDown() {
            mockedLog?.close()
        }
    }

    @Test
    fun testBackPressHandlerCallsUnhandledWhenNativeThrows() {
        var unhandledCalled = false
        val handler = BackPressHandler(onUnhandled = { unhandledCalled = true })

        handler.onBackPressed()

        assertTrue(unhandledCalled)
    }

    @Test
    fun testDeepLinkHandlerCachesPendingLinkWhenUnlinked() {
        val handler = DeepLinkHandler()
        val intent = Mockito.mock(Intent::class.java)
        Mockito.`when`(intent.action).thenReturn(Intent.ACTION_VIEW)
        Mockito.`when`(intent.dataString).thenReturn("moosync://oauth/callback?code=123")

        handler.handleIntent(intent)

        assertEquals("moosync://oauth/callback?code=123", handler.pendingDeepLink)
    }

    @Test
    fun testDeepLinkHandlerClearsPendingLinkOnProcess() {
        val handler = DeepLinkHandler()
        val intent = Mockito.mock(Intent::class.java)
        Mockito.`when`(intent.action).thenReturn(Intent.ACTION_VIEW)
        Mockito.`when`(intent.dataString).thenReturn("moosync://oauth/callback?code=123")
        handler.handleIntent(intent)

        handler.processPendingDeepLink()

        assertNull(handler.pendingDeepLink)
    }

    @Test
    fun testDeepLinkHandlerIgnoresNullIntent() {
        val handler = DeepLinkHandler()

        handler.handleIntent(null)

        assertNull(handler.pendingDeepLink)
    }

    @Test
    fun testDeepLinkHandlerIgnoresNonViewIntent() {
        val handler = DeepLinkHandler()
        val intent = Mockito.mock(Intent::class.java)
        Mockito.`when`(intent.action).thenReturn("android.intent.action.MAIN")
        Mockito.`when`(intent.dataString).thenReturn("moosync://oauth/callback?code=123")

        handler.handleIntent(intent)

        assertNull(handler.pendingDeepLink)
    }
}
