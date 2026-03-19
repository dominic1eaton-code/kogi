import { ComponentFixture, TestBed } from '@angular/core/testing';
import { WalletGroupEconomicsComponent } from './wallet-group-economics.component';

describe('WalletGroupEconomicsComponent', () => {
  let component: WalletGroupEconomicsComponent;
  let fixture: ComponentFixture<WalletGroupEconomicsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [WalletGroupEconomicsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(WalletGroupEconomicsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
