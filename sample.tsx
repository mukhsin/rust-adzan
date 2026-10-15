import React, { useState, useEffect } from 'react';
import {
  Minus,
  Square,
  X,
  Moon,
  Sun,
  MapPin,
  Calendar as CalendarIcon,
  Bell
} from 'lucide-react';

const Mosque = ({ className }) => (
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <path d="M8 22v-4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v4"></path>
    <path d="M12 2v4"></path>
    <path d="M12 6a4 4 0 0 0-4 4v2h8v-2a4 4 0 0 0-4-4Z"></path>
    <path d="M3 12v10"></path>
    <path d="M21 12v10"></path>
    <path d="M3 14h2"></path>
    <path d="M19 14h2"></path>
  </svg>
);

const PRAYER_TIMES = [
  { id: 'imsak', name: 'Imsak', time: '04:05' },
  { id: 'subuh', name: 'Subuh', time: '04:15' },
  { id: 'terbit', name: 'Terbit', time: '05:30' },
  { id: 'dzuhur', name: 'Dzuhur', time: '11:45' },
  { id: 'ashar', name: 'Ashar', time: '14:50' },
  { id: 'maghrib', name: 'Maghrib', time: '17:50' },
  { id: 'isya', name: 'Isya', time: '19:00' },
];

export default function PrayerTimesApp() {
  const [isDark, setIsDark] = useState(true);
  const [currentTime, setCurrentTime] = useState(new Date('2026-10-16T12:53:00')); // Initialized to context time
  const [nextPrayer, setNextPrayer] = useState(PRAYER_TIMES[0]);
  const [countdown, setCountdown] = useState('03:12:00');

  // Simulate clock ticking and countdown
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(prev => new Date(prev.getTime() + 1000));
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Simple countdown logic (mocked for demo purposes to stay relative to our mock times)
  useEffect(() => {
    // In a real app, this would parse the times and calculate exact differences.
    // For this UI demo, we'll just mock a static countdown format that decrements.
    const h = Math.floor((11520000 - (currentTime.getTime() % 11520000)) / 3600000);
    const m = Math.floor((currentTime.getTime() % 3600000) / 60000);
    const s = Math.floor((currentTime.getTime() % 60000) / 1000);

    const formattedCountdown = `03:${(59 - m).toString().padStart(2, '0')}:${(59 - s).toString().padStart(2, '0')}`;
    setCountdown(formattedCountdown);
  }, [currentTime]);

  return (
    <div className={`min-h-screen flex items-center justify-center bg-zinc-100 transition-colors duration-300 ${isDark ? 'dark bg-zinc-950' : ''}`}>

      {/* Desktop Window Container */}
      <div className="w-[380px] shadow-2xl rounded-xl overflow-hidden bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 transition-colors duration-300 flex flex-col font-sans">

        {/* Custom Title Bar */}
        <div className="h-10 bg-zinc-50 dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-800 flex items-center justify-between px-3 select-none cursor-default">
          <div className="flex items-center gap-2">
            <Mosque className="w-4 h-4 text-zinc-600 dark:text-zinc-300" />
            <span className="text-xs font-medium text-zinc-600 dark:text-zinc-300">Jadwal Sholat</span>
          </div>

          <div className="flex items-center gap-1">
            <button
              onClick={() => setIsDark(!isDark)}
              className="p-1.5 rounded-md hover:bg-zinc-200 dark:hover:bg-zinc-800 text-zinc-500 dark:text-zinc-400 transition-colors"
              title="Toggle Theme"
            >
              {isDark ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
            </button>
            <div className="w-px h-4 bg-zinc-300 dark:bg-zinc-700 mx-1"></div>
            <button className="p-1.5 rounded-md hover:bg-zinc-200 dark:hover:bg-zinc-800 text-zinc-500 dark:text-zinc-400 transition-colors">
              <Minus className="w-3.5 h-3.5" />
            </button>
            <button className="p-1.5 rounded-md hover:bg-zinc-200 dark:hover:bg-zinc-800 text-zinc-500 dark:text-zinc-400 transition-colors">
              <Square className="w-3 h-3" />
            </button>
            <button className="p-1.5 rounded-md hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white text-zinc-500 dark:text-zinc-400 transition-colors">
              <X className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* Main Content Area */}
        <div className="flex-1 p-5 flex flex-col gap-6">

          {/* Header Section */}
          <div className="flex flex-col items-center text-center space-y-4 mt-2">
            <div className="flex flex-col gap-1 items-center">
              <div className="flex items-center gap-1.5 text-zinc-500 dark:text-zinc-400 text-sm font-medium">
                <MapPin className="w-4 h-4" />
                <span>Bogor, Indonesia</span>
              </div>
              <div className="flex items-center gap-1.5 text-zinc-400 dark:text-zinc-500 text-xs">
                <CalendarIcon className="w-3.5 h-3.5" />
                <span>Jumat, 16 Oktober 2026</span>
              </div>
            </div>

            <div className="w-full bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-2xl py-6 px-4 flex flex-col items-center justify-center shadow-sm">
              <p className="text-zinc-500 dark:text-zinc-400 text-sm mb-2 font-medium">Menuju {nextPrayer.name}</p>
              <h1 className="text-5xl font-bold tracking-tighter text-zinc-900 dark:text-zinc-50 tabular-nums">
                {countdown}
              </h1>
            </div>
          </div>

          {/* Prayer Times List */}
          <div className="flex flex-col gap-2">
            {PRAYER_TIMES.map((prayer) => {
              const isNext = prayer.id === 'imsak'; // Hardcoded for this demo context

              return (
                <div
                  key={prayer.id}
                  className={`flex items-center justify-between p-3 rounded-xl transition-all duration-200 ${
                    isNext
                      ? 'bg-zinc-900 text-zinc-50 dark:bg-zinc-100 dark:text-zinc-900 shadow-md ring-1 ring-zinc-900/5 dark:ring-white/10 scale-[1.02]'
                      : 'bg-transparent text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-900/50'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <span className={`text-sm ${isNext ? 'font-semibold' : 'font-medium'}`}>
                      {prayer.name}
                    </span>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className={`text-sm tabular-nums tracking-tight ${isNext ? 'font-bold' : 'font-medium text-zinc-500 dark:text-zinc-400'}`}>
                      {prayer.time}
                    </span>
                    <button className={`p-1 rounded-full ${isNext ? 'hover:bg-zinc-800 dark:hover:bg-zinc-200' : 'hover:bg-zinc-200 dark:hover:bg-zinc-800'}`}>
                      <Bell className={`w-3.5 h-3.5 ${isNext ? 'text-zinc-300 dark:text-zinc-600' : 'text-zinc-400 dark:text-zinc-500'}`} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>

        </div>
      </div>
    </div>
  );
}
