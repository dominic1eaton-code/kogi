import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-deal-room',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-deal-room.component.html',
  styleUrl: './marketplace-exchange-deal-room.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeDealRoomComponent {}
